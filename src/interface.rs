use alloy::sol;

sol!(
    #[sol(rpc)]
    contract IStabilizer {
        event LiquidityAdded(uint256 amountUsdc, uint256 amountUsdt, uint256 amountStb, address receiver);
        event LiquidityRemoved(uint256 amountUsdc, uint256 amountUsdt, uint256 amountStb, address receiver);
        event Exchange(
            address token, uint256 amount, uint256 quoteAmount, uint256 fees, address receiver, address feeReceiver
        );
        function getStabilizerMatrix()
            external
            returns (uint256 usdcReserveAmount, uint256 usdtReserveAmount, uint256 usdcPrice, uint256 usdtPrice);
    }
);
