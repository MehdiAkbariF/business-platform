use async_trait::async_trait;
use application::errors::AppError;
use application::ports::monetization::PaymentProviderPort;
use domain::monetization::Money;
use shared::PaymentId;

pub struct MockPaymentProvider;

#[async_trait]
impl PaymentProviderPort for MockPaymentProvider {
    async fn initiate_payment(&self, payment_id: PaymentId, _amount: Money, _callback: &str) -> Result<String, AppError> {
        Ok(format!("https://payment-gateway.mock/pay/{}", payment_id.0))
    }

    async fn verify_payment(&self, _provider_ref: &str, _amount: Money) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn refund_payment(&self, _provider_ref: &str, _amount: Money) -> Result<bool, AppError> {
        Ok(true)
    }
}