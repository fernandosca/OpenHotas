use crate::tuning::DEFAULT_MAX_JUMP;
use libm::fabsf;

/// Filtro de rejeição de spikes — descarta amostras que mudem abruptamente.
///
/// # Posição no pipeline
///
/// Calibration → **MaxJump** → EMA → Deadzone → ResponseCurve
///
/// É o PRIMEIRO filtro depois da calibração. Deve vir antes do EMA porque:
/// 1. Um spike passageiro corromperia a média do EMA por vários ciclos.
/// 2. O MaxJump age amostra-a-amostra, sem memória além do último valor.
///
/// # Invariantes
///
/// - `threshold` é armazenado como valor absoluto (`fabsf`).
/// - `initialized = false` até o primeiro `apply()`; o primeiro sample
///   é sempre aceito para estabelecer a referência inicial.
/// - Se o delta entre o sample atual e `last_valid` excede `threshold`,
///   o sample é rejeitado e o último valor válido é mantido.
///
/// # Comportamento esperado
///
/// - Spike curto (1 sample) > threshold → descartado, continua emitindo `last_valid`.
/// - Mudança legítima sustentada: o primeiro sample da mudança é rejeitado,
///   mas o segundo (igual ao primeiro) tem delta 0 e é aceito, atualizando
///   `last_valid`. Isso significa que o filtro atrasa a transição em 1 ciclo.
/// - `threshold` em unidades normalizadas [-1..1]. Escalado pelo span da
///   calibração em `from_protocol_config` (V1.24).
///
/// # Modo de falha
///
/// Se um pino/trace do encoder estiver ruidoso (bit flutuante), o filtro
/// pode rejeitar continuamente e travar o eixo no último valor válido.
/// Não há timeout para destravar — o eixo fica "travado" até que um sample
/// dentro do threshold chegue. Não tratado atualmente.
///
/// # no_std / heap
///
/// Sem heap. Apenas flags + aritmética f32.
#[derive(Debug, Clone)]
pub struct MaxJump {
    /// Último valor que passou pelo filtro.
    last_valid: f32,
    /// `false` até o primeiro sample; `true` depois.
    initialized: bool,
    /// Delta máximo tolerado entre amostras consecutivas (valor absoluto).
    threshold: f32,
}

impl MaxJump {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: fabsf(threshold),
            last_valid: 0.0,
            initialized: false,
        }
    }

    /// Processa um sample: aceita se delta <= threshold, rejeita se >.
    /// No primeiro sample (ou após `reset()`), aceita sem comparar.
    pub fn apply(&mut self, input: f32) -> f32 {
        if !self.initialized {
            self.last_valid = input;
            self.initialized = true;
            return input.clamp(-1.0, 1.0);
        }

        let delta = fabsf(input - self.last_valid);
        if delta > self.threshold {
            return self.last_valid;
        }

        self.last_valid = input;
        input.clamp(-1.0, 1.0)
    }

    /// Altera o threshold em runtime (valor absoluto).
    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = fabsf(t);
    }

    /// Reseta o estado interno — o próximo `apply()` será tratado como inicial.
    /// Usado quando calibração ou travel limits mudam (V1.25).
    pub fn reset(&mut self) {
        self.initialized = false;
    }
}

impl Default for MaxJump {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_JUMP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_input_accepted() {
        let mut mj = MaxJump::new(0.1);
        assert_eq!(mj.apply(0.5), 0.5);
    }

    #[test]
    fn within_threshold_accepted() {
        let mut mj = MaxJump::new(0.1);
        mj.apply(0.5);
        assert_eq!(mj.apply(0.55), 0.55);
    }

    #[test]
    fn spike_rejected_holds_last() {
        let mut mj = MaxJump::new(0.1);
        mj.apply(0.5);
        assert_eq!(mj.apply(0.9), 0.5);
    }

    #[test]
    fn negative_spike_rejected() {
        let mut mj = MaxJump::new(0.1);
        mj.apply(0.5);
        assert_eq!(mj.apply(0.1), 0.5);
    }
}
