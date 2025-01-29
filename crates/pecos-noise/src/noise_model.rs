use pecos_core::types::CommandBatch;

/// Trait defining interface for quantum noise models
pub trait NoiseModel: Send + Sync {
    /// Apply noise to a batch of quantum commands
    fn apply_noise(&self, commands: CommandBatch) -> CommandBatch;
}
