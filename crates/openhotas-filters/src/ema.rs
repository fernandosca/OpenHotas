use crate::tuning::DEFAULT_EMA_ALPHA;

/// Exponential Moving Average — suavização da saída do MaxJump.
///
/// # Posição no pipeline
///
/// Calibration → MaxJump → **EMA** → Deadzone → ResponseCurve
///
/// Aplica-se DEPOIS do MaxJump (spike rejection) e ANTES da Deadzone.
/// Ordem intencional: ruído residual rejeitado pelo MaxJump ainda existe;
/// o EMA suaviza antes da deadzone cortar o sinal perto de centro,
/// evitando transições abruptas na borda da zona morta.
///
/// # Invariantes
///
/// - `alpha` é clamped em [0.0, 1.0] no construtor e em `set_alpha`.
/// - `input` é clamped em [-1.0, 1.0] a cada `apply()`.
/// - `initialized = false` na criação e após `reset()`; o primeiro `apply()`
///   copia o sample sem filtragem (evita transiente do valor inicial 0.0).
/// - `value` é sempre mantido em [-1.0, 1.0].
///
/// # Comportamento nos extremos de alpha
///
/// - `alpha = 0.0`: mantém o último valor indefinidamente (hold mode).
/// - `alpha = 1.0`: pass-through, sem suavização.
/// - `alpha` entre 0.0 e 1.0: compromisso entre suavidade e latência.
///
/// # no_std / heap
///
/// Sem heap. Apenas aritmética f32 + flags.
#[derive(Debug, Clone)]
pub struct Ema {
    /// Fator de suavização (0..1). Quanto menor, mais suave (mais latente).
    alpha: f32,
    /// Valor filtrado atual.
    value: f32,
    /// `false` após criação ou reset; `true` após o primeiro sample.
    /// Evita que `value` inicial 0.0 produza um transiente no primeiro ciclo.
    initialized: bool,
}

impl Ema {
    pub fn new(alpha: f32) -> Self {
        let clamped = alpha.clamp(0.0, 1.0);
        Self {
            alpha: clamped,
            value: 0.0,
            initialized: false,
        }
    }

    /// Processa um sample através do filtro EMA.
    ///
    /// No primeiro sample (ou após `reset()`), retorna o input sem filtragem.
    /// Nos demais, retorna `alpha * input + (1 - alpha) * last_value`.
    /// Saída sempre clamped em [-1.0, 1.0].
    pub fn apply(&mut self, input: f32) -> f32 {
        let input = input.clamp(-1.0, 1.0);
        if !self.initialized {
            self.value = input;
            self.initialized = true;
            return input;
        }
        self.value = self.alpha * input + (1.0 - self.alpha) * self.value;
        self.value.clamp(-1.0, 1.0)
    }

    /// Reseta o filtro: o próximo `apply()` tratará o sample como inicial.
    /// Usado quando calibração ou travel limits mudam (V1.25).
    pub fn reset(&mut self) {
        self.initialized = false;
    }

    /// Altera o fator de suavização em runtime.
    /// O novo alpha é clamped em [0.0, 1.0].
    pub fn set_alpha(&mut self, a: f32) {
        self.alpha = a.clamp(0.0, 1.0);
    }
}

impl Default for Ema {
    fn default() -> Self {
        Self::new(DEFAULT_EMA_ALPHA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sample_initializes() {
        let mut ema = Ema::new(0.3);
        assert_eq!(ema.apply(0.5), 0.5);
    }

    #[test]
    fn smoothing_known_alpha() {
        let mut ema = Ema::new(0.5);
        ema.apply(0.0);
        let result = ema.apply(1.0);
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn converges_to_target() {
        let mut ema = Ema::new(0.3);
        ema.apply(0.0);
        for _ in 0..30 {
            ema.apply(1.0);
        }
        let result = ema.apply(1.0);
        assert!(
            (result - 1.0).abs() < 0.001,
            "EMA did not converge: {}",
            result
        );
    }

    #[test]
    fn reset_forces_reinit() {
        let mut ema = Ema::new(0.3);
        ema.apply(1.0);
        ema.reset();
        assert_eq!(ema.apply(0.0), 0.0);
    }

    #[test]
    fn alpha_zero_holds() {
        let mut ema = Ema::new(0.0);
        ema.apply(1.0);
        assert_eq!(ema.apply(2.0), 1.0);
    }

    #[test]
    fn alpha_one_passthrough() {
        let mut ema = Ema::new(1.0);
        assert_eq!(ema.apply(0.5), 0.5);
        assert_eq!(ema.apply(0.8), 0.8);
    }
}
