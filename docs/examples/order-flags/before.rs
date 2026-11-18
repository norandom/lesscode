// Made up for the introduction. An order whose state is three flags.
#[derive(Default)]
pub struct Order {
    paid: bool,
    shipped: bool,
    cancelled: bool,
}

impl Order {
    pub fn pay(&mut self) -> Result<(), &'static str> {
        if self.cancelled {
            return Err("cancelled");
        }
        if self.paid {
            return Err("already paid");
        }
        self.paid = true;
        Ok(())
    }

    pub fn ship(&mut self) -> Result<(), &'static str> {
        if self.cancelled {
            return Err("cancelled");
        }
        if !self.paid {
            return Err("not paid");
        }
        if self.shipped {
            return Err("already shipped");
        }
        self.shipped = true;
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), &'static str> {
        if self.shipped {
            return Err("already shipped");
        }
        if self.cancelled {
            return Err("cancelled");
        }
        self.cancelled = true;
        Ok(())
    }

    pub fn label(&self) -> &'static str {
        if self.cancelled {
            if self.paid { "refund due" } else { "cancelled" }
        } else if self.shipped {
            "shipped"
        } else if self.paid {
            "paid"
        } else {
            "open"
        }
    }
}
