#[derive(Debug, Clone, Copy)]
pub struct ProposalOptions {
    pub explicit_resource_management: bool,
}

impl Default for ProposalOptions {
    fn default() -> Self {
        Self { explicit_resource_management: true }
    }
}
