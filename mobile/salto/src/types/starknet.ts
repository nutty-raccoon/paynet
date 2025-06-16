export type StarknetCall = {
  to: string,
  selector: string,
  calldata: string[],
}

export type StarknetPaymentRequest = StarknetCall[];
