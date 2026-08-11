use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("cleanup plan contains no candidates")]
    EmptyPlan,
    #[error("cleanup plan has expired")]
    ExpiredPlan,
    #[error("cleanup plan is not in the prepared state")]
    PlanNotPrepared,
}
