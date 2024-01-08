import { OrderAsset } from "./order-asset"

export interface Order {
    hash: string;
    chainId: number;
    createdAt: number;
    type: OrderType;
    status: OrderStatus;
    input: OrderAsset;
    output: OrderAsset;
    fee: OrderAsset | null;
    recipient: string;
    signature: string;
    tx: string | null;
}

export type OrderType = "Dutch" | "DutchLimit";

export type OrderStatus = "open" | "filled" | "cancelled" | "expired" | "error" | "insufficient-funds";