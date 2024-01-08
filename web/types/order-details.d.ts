export interface OrderDetails {
    decayStartTime: number;
    decayEndTime: number;
    exclusiveFiller: string;
    exclusivityOverrideBps: number;
    reactor: string;
    swapper: string;
    nonce: string;
    deadline: number;
    additionalValidationContract: string;
    additionalValidationData: string;
}