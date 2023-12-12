use stapifaction::Persistable;

#[derive(Debug, Persistable)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetails {
    pub decay_start_time: u64,
    pub decay_end_time: u64,
    pub exclusive_filler: String,
    pub exclusivity_override_bps: u64,
    pub reactor: String,
    pub swapper: String,
    pub nonce: String,
    pub deadline: u64,
    pub additional_validation_contract: String,
    pub additional_validation_data: String,
}
