export interface OrderAsset {
    startAmount: number;
    endAmount: number;
    settledAmount: number | null;
    symbol: string;
    price: number;
}