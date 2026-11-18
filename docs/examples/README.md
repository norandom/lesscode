# Examples: from scattered state to one state machine

## An opener, made up

This first example is invented to show the idea in one screen. The examples below it are real code.

An order can be paid, shipped and cancelled. The first version keeps that as three flags, and each
method asks the flags what is allowed:

```rust
pub struct Order {
    paid: bool,
    shipped: bool,
    cancelled: bool,
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
// pay, cancel and label look the same: eleven `if`s in all
```

Three flags can be set in eight ways. Only five of them mean anything. Nothing in the code says
which five, and "cancelled and paid" turns out to mean "refund due" only in the method that prints
the label. To learn what an order can do, you read every method.

The second version says it once:

```rust
pub enum Order {
    Open,
    Paid,
    Shipped,
    Cancelled,
    RefundDue,
}

pub fn apply(self, event: Event) -> Result<Order, &'static str> {
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
```

| | Flags | One value |
| --- | --- | --- |
| Places that hold the order's state | 3 | 1 |
| States that can be written down | 8 | 5 |
| States that mean something | 5 | 5 |
| Where the rules are | 11 `if`s in 4 methods | 1 table of 8 rows |

Measured with `kiro-cli`, both versions hold 11 decisions. The largest routine has 5 before and 8
after: the decisions moved into one place.

The table still decides eight cases; the rules did not get fewer. They are in one place, and a
state that means nothing can no longer be written.

Both versions answer alike: [check.rs](order-flags/check.rs) runs every sequence of up to six
events (6,015 steps) through both and compares the result and the label after each step. Full
sources: [before.rs](order-flags/before.rs), [after.rs](order-flags/after.rs).

Real code is rarely this tidy. The flags sit in different files and go by other names.
Finding them is the hard part, and that is what the research on this page's real examples is about.

## Real examples

How to do this yourself, with the commands: [Untangling scattered state](../state-refactoring.md).

| Example | Code | What changed |
| --- | --- | --- |
| [Batch progress](batch-progress/README.md) | this repository, the batch runner of `lesscode-core` | Three locked fields became one value; judged and accepted by the tool's own step flow, after it refused a wrong declaration |
| [Reply side channel](reply-side-channel/README.md) | this repository, the repair loop of `lesscode-model` | A cell that carried one fact between two closures is gone; one rule moved from one caller into the loop, for all three callers |
| [Clipboard lease](clipboard-lease/README.md) | `drift-rdp`, a remote desktop client | Three cells that held one machine became one value; conditions 5 to 3; 416 tests pass before and after |

## Earlier

- [Self-evaluation](self-evaluation.md): archived; describes the retired Python prototype.
