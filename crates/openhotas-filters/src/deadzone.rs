use crate::tuning::DEFAULT_DEADZONE;
use libm::fabsf;

/// Deadzone — zera o sinal perto do centro e remapeia o restante para [-1, 1].
///
/// # Posição no pipeline
///
/// Calibration → MaxJump → EMA → **Deadzone** → ResponseCurve
///
/// Vem DEPOIS do EMA porque queremos que a suavização ocorra antes de
/// cortar o sinal na borda da zona morta. Se a deadzone viesse antes,
/// o EMA suavizaria a transição abrupta de 0→nonzero na saída da zona,
/// introduzindo latência indesejada no início do movimento.
///
/// # Invariantes
///
/// - `threshold` clamped em [0.0, 1.0].
/// - `in_zone` rastreia se o sinal está DENTRO da zona morta no ciclo anterior.
///   Usado para detectar a transição de fora→dentro (`just_entered`).
///
/// # Remapeamento
///
/// Valores dentro de [-threshold, +threshold] → 0.0.
/// Valores fora são reescalados linearmente para [-1.0, 1.0]:
///
/// ```text
/// output = sign(input) * (|input| - threshold) / (1 - threshold)
/// ```
///
/// Isso preserva a inclinação da curva de resposta após a deadzone,
/// evitando que o usuário precise de mais movimento para atingir 100%.
///
/// # Flag `just_entered`
///
/// A tupla de retorno inclui `(valor_filtrado, entrou_na_zona)`.
/// `entrou_na_zona = true` apenas no primeiro ciclo em que o sinal cruza
/// para dentro da deadzone. Usado pelo `AxisPipeline` para resetar o EMA
/// no eixo Twist, mas NÃO nos eixos X/Y (preserva continuidade perto do centro).
///
/// # Casos extremos
///
/// - `threshold = 0.0`: pass-through, sem zona morta.
/// - `threshold >= 1.0`: toda a faixa é zona morta — saída sempre 0.0.
///
/// # no_std / heap
///
/// Sem heap. Flags + aritmética f32.
#[derive(Debug)]
pub struct Deadzone {
    /// Limiar da zona morta em unidades normalizadas [0.0, 1.0].
    threshold: f32,
    /// `true` se o sample anterior estava dentro da zona morta.
    /// Usado para detectar a transição de entrada (just_entered).
    in_zone: bool,
}

impl Deadzone {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            in_zone: false,
        }
    }

    /// Aplica a zona morta. Retorna `(valor_processado, entrou_na_zona)`.
    ///
    /// `entrou_na_zona` é `true` apenas no ciclo em que o sinal cruza
    /// de fora para dentro da zona morta. Usado para reset de EMA.
    /// NaN é tratado como 0.0 (centro), sem alterar `in_zone`.
    pub fn apply(&mut self, input: f32) -> (f32, bool) {
        // NaN bypassa clamp — trata como centro, sem transição de zona.
        if input.is_nan() {
            return (0.0, false);
        }
        let input = input.clamp(-1.0, 1.0);
        let mut just_entered = false;

        if fabsf(input) < self.threshold {
            if !self.in_zone {
                just_entered = true;
            }
            self.in_zone = true;
            return (0.0, just_entered);
        }

        self.in_zone = false;

        if self.threshold >= 1.0 {
            return (input, just_entered);
        }

        let sign = if input >= 0.0 { 1.0 } else { -1.0 };
        let val = sign * (fabsf(input) - self.threshold) / (1.0 - self.threshold);
        (val, just_entered)
    }

    /// Altera o limiar da zona morta em runtime.
    /// O novo valor é clamped em [0.0, 1.0].
    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = t.clamp(0.0, 1.0);
    }
}

impl Default for Deadzone {
    fn default() -> Self {
        Self::new(DEFAULT_DEADZONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_zeroed() {
        let mut dz = Deadzone::new(0.1);
        let (out, _) = dz.apply(0.05);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn above_threshold_remapped() {
        let mut dz = Deadzone::new(0.1);
        let (out, _) = dz.apply(0.55);
        assert!((out - 0.5).abs() < 0.001);
    }

    #[test]
    fn just_entered_flag() {
        let mut dz = Deadzone::new(0.1);
        dz.apply(0.5);
        let (_, entered) = dz.apply(0.05);
        assert!(entered);
    }

    #[test]
    fn negative_side_mirrors() {
        let mut dz = Deadzone::new(0.1);
        let (out, _) = dz.apply(-0.55);
        assert!((out - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn zero_threshold_passthrough() {
        let mut dz = Deadzone::new(0.0);
        let (out, _) = dz.apply(0.75);
        assert!((out - 0.75).abs() < 0.001);
    }
}
