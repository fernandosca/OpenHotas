//! Calibração de eixos para sensor MT6826S (15-bit absoluto, 0..32767).
//!
//! # Topologia do sensor
//!
//! O MT6826S é um encoder magnético absoluto de 15 bits: 0..32767 (32768 posições).
//! O valor cru (`raw`) é uma posição angular no círculo de 15 bits.
//! A calibração mapeia três pontos (min, center, max) para [-1.0, 0.0, +1.0].
//!
//! # Circularidade
//!
//! Como o sensor é um círculo, o delta entre raw e center precisa ser a
//! distância mais curta no círculo — daí `circular_delta`. Isso garante que
//! a transição 32767 → 0 (cruzamento do zero físico) seja contínua para
//! qualquer eixo com curso menor que meia revolução.
//!
//! # Pipeline position
//!
//! Calibration é o PRIMEIRO estágio do pipeline:
//! **Calibration** → MaxJump → EMA → Deadzone → ResponseCurve
//!
//! # no_std / heap
//!
//! Sem heap. Apenas aritmética i32/f32.

// DEFAULT_CENTER = 16384 = metade de 32768 (centro teórico do range de 15 bits).
// Este valor é o default do sensor MT6826S quando montado no centro físico.
// O usuário pode redefinir via calibração. NÃO ALTERAR sem recalibrar todos os sticks.
const DEFAULT_CENTER: u16 = 16384;
// DEFAULT_MAX = 32767 = valor máximo do encoder de 15 bits.
const DEFAULT_MAX: u16 = 32767;
// SENSOR_COUNTS = 32768 = número total de posições únicas (0..32767).
// É uma potência de 2 (2^15), permitindo a máscara bitwise no circular_delta.
const SENSOR_COUNTS: i32 = 32768;
const HALF_TURN: i32 = SENSOR_COUNTS / 2;

/// Dados brutos de calibração: três pontos no range do sensor (0..32767).
///
/// center: posição do centro físico (joystick em repouso).
/// min: posição no fim de curso em uma direção.
/// max: posição no fim de curso na direção oposta.
///
/// Invariante: min e max devem estar em lados opostos do center, e o span
/// total (distância center→min + center→max) deve ser >= minimum_span.
///
/// Tanto min > center quanto min < center são válidos — o algoritmo detecta
/// automaticamente a direção do sensor.
#[derive(Debug, Clone, Copy)]
pub struct CalibrationData {
    pub center: u16,
    pub min: u16,
    pub max: u16,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            center: DEFAULT_CENTER,
            min: 0,
            max: DEFAULT_MAX,
        }
    }
}

impl CalibrationData {
    /// Menor distância signed de `center` a `raw` no círculo de 15 bits.
    ///
    /// O resultado está sempre em -16384..=16383, então a transição 32767→0
    /// é contínua para qualquer eixo com curso < meia revolução.
    ///
    /// Exemplo: center=1000, raw=30000 → delta = -2768 (caminho curto
    /// passando por 32767→0 em vez de +29000).
    ///
    /// Implementação: subtrai, adiciona HALF_TURN, mascara (SENSOR_COUNTS-1),
    /// subtrai HALF_TURN de volta. Isso é equivalente a `(raw - center)
    /// mod SENSOR_COUNTS` tratado como signed no range [-HALF_TURN, HALF_TURN).
    pub fn circular_delta(raw: u16, center: u16) -> i32 {
        ((raw as i32 - center as i32 + HALF_TURN) & (SENSOR_COUNTS - 1)) - HALF_TURN
    }

    /// Distância signed do center ao min (sempre negativo se min < center,
    /// positivo se min > center — mas `is_valid` exige lados opostos).
    pub fn min_delta(&self) -> i32 {
        Self::circular_delta(self.min, self.center)
    }

    /// Distância signed do center ao max.
    pub fn max_delta(&self) -> i32 {
        Self::circular_delta(self.max, self.center)
    }

    /// Curso total calibrado em raw counts, independente de zero crossing.
    /// Soma as distâncias absolutas de center a min e center a max.
    pub fn span(&self) -> u32 {
        self.min_delta().unsigned_abs() + self.max_delta().unsigned_abs()
    }

    /// Valida se os pontos de calibração são utilizáveis.
    ///
    /// Critérios:
    /// 1. min_delta != 0 (min diferente de center)
    /// 2. max_delta != 0 (max diferente de center)
    /// 3. min e_max em lados opostos de center (signum diferentes)
    /// 4. span >= minimum_span (rejeita capturas acidentais com curso mínimo)
    ///
    /// O `minimum_span` padrão usado no firmware é 1000 (valores empíricos,
    /// garante que o usuário moveu o stick o suficiente durante calibração).
    pub fn is_valid(&self, minimum_span: u32) -> bool {
        let min = self.min_delta();
        let max = self.max_delta();
        min != 0 && max != 0 && min.signum() != max.signum() && self.span() >= minimum_span
    }
}

/// Wrapper runtime que aplica CalibrationData a amostras cruas do sensor.
///
/// # Degradação
///
/// Se `is_valid` falhar (ex: calibração corrompida), `apply` retorna 0.0.
/// Isso significa que o eixo fica centrado e não responde a movimento —
/// comportamento deliberado para evitar saídas espúrias com dados inválidos.
#[derive(Debug)]
pub struct Calibration {
    pub data: CalibrationData,
}

impl Calibration {
    pub fn new(data: CalibrationData) -> Self {
        Self { data }
    }

    /// Aplica a calibração a um valor cru do sensor.
    ///
    /// Retorna:
    /// - 0.0 se raw == center ou se a calibração é inválida
    /// - [-1.0, 0.0) se raw está no lado do min
    /// - (0.0, 1.0] se raw está no lado do max
    ///
    /// Divisão por min_delta ou max_delta: segura porque `is_valid` garante
    /// que ambos são não-nulos e de sinais opostos.
    pub fn apply(&self, raw: u16) -> f32 {
        let delta = CalibrationData::circular_delta(raw, self.data.center);
        let min_delta = self.data.min_delta();
        let max_delta = self.data.max_delta();

        // Se delta == 0, raw == center → saída 0.0.
        // Se calibração inválida (is_valid false), retorna 0.0 (degradado).
        if delta == 0 || !self.data.is_valid(1) {
            return 0.0;
        }

        // Determina de que lado do center o raw está:
        // Se tem o mesmo sinal de min_delta → lado do min (negativo).
        // Senão → lado do max (positivo).
        if delta.signum() == min_delta.signum() {
            (-(delta as f32 / min_delta as f32)).clamp(-1.0, 0.0)
        } else {
            (delta as f32 / max_delta as f32).clamp(0.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_returns_zero() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(16384), 0.0);
    }

    #[test]
    fn min_returns_neg_one() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(0), -1.0);
    }

    #[test]
    fn max_returns_pos_one() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(32767), 1.0);
    }

    #[test]
    fn degenerate_range_no_panic() {
        let cal = CalibrationData {
            center: 16384,
            min: 16384,
            max: 16384,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(16384), 0.0);
    }

    #[test]
    fn calibration_crosses_zero() {
        let cal = CalibrationData {
            min: 30000,
            center: 1000,
            max: 4000,
        };
        let wrapper = Calibration::new(cal);

        assert!(cal.is_valid(1000));
        assert_eq!(wrapper.apply(30000), -1.0);
        assert_eq!(wrapper.apply(1000), 0.0);
        assert_eq!(wrapper.apply(4000), 1.0);
        assert!(wrapper.apply(32700) < 0.0);
        assert!(wrapper.apply(2000) > 0.0);
    }

    #[test]
    fn reversed_sensor_direction_crosses_zero() {
        let cal = CalibrationData {
            min: 4000,
            center: 1000,
            max: 30000,
        };
        let wrapper = Calibration::new(cal);

        assert!(cal.is_valid(1000));
        assert_eq!(wrapper.apply(4000), -1.0);
        assert_eq!(wrapper.apply(1000), 0.0);
        assert_eq!(wrapper.apply(30000), 1.0);
    }

    #[test]
    fn endpoints_on_same_side_are_invalid() {
        let cal = CalibrationData {
            min: 1200,
            center: 1000,
            max: 4000,
        };
        assert!(!cal.is_valid(1));
    }

    #[test]
    fn asymmetric_sides() {
        let cal = CalibrationData {
            center: 20000,
            min: 10000,
            max: 30000,
        };
        let wrapper = Calibration::new(cal);
        assert_eq!(wrapper.apply(10000), -1.0);
        assert_eq!(wrapper.apply(30000), 1.0);
    }

    #[test]
    fn symmetric_center() {
        let cal = CalibrationData {
            center: 16384,
            min: 0,
            max: 32767,
        };
        let wrapper = Calibration::new(cal);
        assert!((wrapper.apply(8192) - (-0.5)).abs() < 0.001);
        assert!((wrapper.apply(24576) - 0.5).abs() < 0.001);
    }
}
