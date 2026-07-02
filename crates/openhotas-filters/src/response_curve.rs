/// Curva de resposta piecewise linear com 5 pontos de controle.
///
/// # Posição no pipeline
///
/// Calibration → MaxJump → EMA → Deadzone → **ResponseCurve**
///
/// É o ÚLTIMO filtro da cadeia. Age sobre o sinal já calibrado,
/// suavizado e com zona morta aplicada — modela a sensibilidade
/// final do eixo (ex: expo, curva S, resposta linear).
///
/// # Controle da curva
///
/// P0 = (-1, -1), P2 = (0, 0), P4 = (1, 1) são fixos.
/// P1 (point_left, lado negativo) e P3 (point_right, lado positivo)
/// são configuráveis:
/// - P1.x ∈ [-1.0, 0.0] — sempre à esquerda do centro
/// - P3.x ∈ [0.0, 1.0] — sempre à direita do centro
/// - P1.y, P3.y ∈ [-1.0, 1.0]
///
/// O usuário define P1 e P3 via GUI (response_p1, response_p3 no protocolo).
/// O protocolo escala de permille para f32 em `from_protocol_config`.
///
/// # Por que piecewise linear?
///
/// Escolha deliberada vs spline/exponencial: executa em O(4) sem heap,
/// sem lookup table, sem funções transcendentais (`libm::powf`). O RP2350
/// não tem FPU de hardware dedicada para essas operações.
/// A curva resultante é contínua mas tem derivada descontínua nos pontos.
///
/// # Invariantes
///
/// - `points` está sempre ordenado por X crescente: -1 ≤ P1.x ≤ 0 ≤ P3.x ≤ 1.
/// - A saída é clamped em [-1.0, 1.0] em cada segmento.
/// - Se um segmento tem dx ~ 0 (pontos coincidentes em X), o fallback
///   `dx.abs() < f32::EPSILON` retorna `y0` para evitar divisão por zero.
///
/// # Modo de falha
///
/// Se P1 e P2 (ou P2 e P3) tiverem o mesmo X, o segmento é degenerado
/// e o valor de saída é `y0` constante naquele trecho — não quebra.
///
/// # no_std / heap
///
/// Sem heap. Apenas aritmética f32.
#[derive(Debug, Clone, Copy)]
pub struct ResponseCurve {
    /// 5 pontos de controle: [P0, P1, P2, P3, P4]
    points: [(f32, f32); 5],
}

impl ResponseCurve {
    /// Cria uma nova curva com os pontos P1 e P3 configuráveis.
    /// P0, P2, P4 são fixos; P1 e P3 são clamped nos ranges válidos.
    pub fn new(p1: (f32, f32), p3: (f32, f32)) -> Self {
        Self {
            points: [
                (-1.0, -1.0),
                (p1.0.clamp(-1.0, 0.0), p1.1.clamp(-1.0, 1.0)),
                (0.0, 0.0),
                (p3.0.clamp(0.0, 1.0), p3.1.clamp(-1.0, 1.0)),
                (1.0, 1.0),
            ],
        }
    }

    /// Aplica a curva de resposta a um valor normalizado [-1, 1].
    /// Interpola linearmente entre os pontos de controle.
    pub fn apply(&self, input: f32) -> f32 {
        let input = input.clamp(-1.0, 1.0);

        for i in 0..4 {
            let (x0, y0) = self.points[i];
            let (x1, y1) = self.points[i + 1];
            if input <= x1 {
                let dx = x1 - x0;
                // Segmento degenerado: pontos coincidem em X.
                // Em vez de dividir por zero, retorna y0.
                if dx.abs() < f32::EPSILON {
                    return y0.clamp(-1.0, 1.0);
                }
                let t = (input - x0) / dx;
                return (y0 + t * (y1 - y0)).clamp(-1.0, 1.0);
            }
        }

        // input > P4.x (só acontece se clamp falhar — defensivo)
        1.0
    }

    /// Atualiza os pontos P1 e P3 em runtime.
    pub fn set_points(&mut self, p1: (f32, f32), p3: (f32, f32)) {
        *self = Self::new(p1, p3);
    }
}

impl Default for ResponseCurve {
    fn default() -> Self {
        // Default: P1 = (-0.5, -0.5), P3 = (0.5, 0.5) — curva linear.
        Self::new((-0.5, -0.5), (0.5, 0.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_default() {
        let curve = ResponseCurve::default();
        assert!((curve.apply(-1.0) - (-1.0)).abs() < f32::EPSILON);
        assert!((curve.apply(0.0)).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_output() {
        let curve = ResponseCurve::default();
        assert!(curve.apply(-1.5) >= -1.0);
        assert!(curve.apply(1.5) <= 1.0);
    }

    #[test]
    fn segment_interpolation() {
        let curve = ResponseCurve::new((-0.5, -0.8), (0.5, 0.8));
        let mid_left = curve.apply(-0.25);
        let mid_right = curve.apply(0.25);
        assert!(mid_left < -0.25);
        assert!(mid_right > 0.25);
    }

    #[test]
    fn set_points_roundtrip() {
        let mut rc = ResponseCurve::default();
        rc.set_points((-0.5, -1.0), (0.5, 1.0));
        assert!((rc.apply(0.5) - 1.0).abs() < f32::EPSILON);
    }
}
