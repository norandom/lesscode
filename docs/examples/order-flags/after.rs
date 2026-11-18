// Made up for the introduction. The same order as one value and one table.
#[derive(Clone, Copy, Default)]
pub enum Order {
    #[default]
    Open,
    Paid,
    Shipped,
    Cancelled,
    RefundDue,
}

pub enum Event {
    Pay,
    Ship,
    Cancel,
}

impl Order {
    pub fn apply(self, event: Event) -> Result<Order, &'static str> {
        use {Event::*, Order::*};
        match (self, event) {
            (Open, Pay) => Ok(Paid),
            (Paid, Ship) => Ok(Shipped),
            (Open, Cancel) => Ok(Cancelled),
            (Paid, Cancel) => Ok(RefundDue),
            (Cancelled | RefundDue, _) => Err("cancelled"),
            (Paid | Shipped, Pay) => Err("already paid"),
            (Open, Ship) => Err("not paid"),
            (Shipped, Ship | Cancel) => Err("already shipped"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Order::Open => "open",
            Order::Paid => "paid",
            Order::Shipped => "shipped",
            Order::Cancelled => "cancelled",
            Order::RefundDue => "refund due",
        }
    }
}
