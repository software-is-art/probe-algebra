//! money — a spike on the tagged-primitive value-object substrate. See
//! `money::boundary` and `crate::boundary::Qty`. It shows the discipline ("every
//! primitive is a value object with operators") paid for at a fraction of the
//! boilerplate: a kind tag plus, at most, a validity rule.

pub mod boundary;

#[cfg(test)]
mod tests {
    use crate::boundary::Pair;
    use crate::money::boundary::{Balance, Cents, Points};

    /// A full-domain named concept gets total arithmetic with NO per-type
    /// operators — `add`/`sub`/`zero`/`negate` come from the generic `Qty<Total>`.
    #[test]
    fn full_domain_concept_has_free_total_arithmetic() {
        let a = Balance::new(100).unwrap();
        let b = Balance::new(250).unwrap();
        assert_eq!(a.plus(b).get(), 350);
        assert_eq!(a.minus(b).get(), -150);
        assert_eq!(Balance::zero().get(), 0);
        assert_eq!(a.negate().get(), -100);
    }

    /// A partitioning concept gets CHECKED arithmetic from the SAME generic code,
    /// honouring its validity rule — still no hand-written operators.
    #[test]
    fn partitioning_concept_has_free_checked_arithmetic() {
        let c = Cents::new(100).unwrap();
        assert_eq!(c.checked_add(Cents::new(50).unwrap()), Cents::new(150));
        assert!(Cents::new(200_000_000).is_none()); // outside the partition
        let big = Cents::new(100_000_000).unwrap();
        assert!(big.checked_add(big).is_none()); // the sum leaves the partition
    }

    /// Unit safety: distinct kinds over the same primitive do NOT unify — `Balance`
    /// and `Points` are different types even though both are full-domain `i64`.
    #[test]
    fn distinct_kinds_do_not_unify() {
        let bal = Balance::new(10).unwrap();
        let pts = Points::new(10).unwrap();
        assert_eq!(bal.get(), pts.get()); // same underlying value...
                                          // ...but distinct types: `bal.add(pts)` would not compile.
        assert_ne!(
            core::any::type_name::<Balance>(),
            core::any::type_name::<Points>()
        );
    }

    /// Tagged values are first-class value objects: they compose with the grammar
    /// (here a residual `Pair`) with no special-casing.
    #[test]
    fn tagged_values_are_value_objects() {
        let p = Pair(Balance::new(1).unwrap(), Cents::new(2).unwrap());
        assert_eq!(p.0.get(), 1);
        assert_eq!(p.1.get(), 2);
    }

    /// The remaining generic surface: checked subtraction (honouring the
    /// partition), ordering, and Debug — all provided once by `Qty`.
    #[test]
    fn generic_surface_is_exercised() {
        let c = Cents::new(100).unwrap();
        assert_eq!(c.checked_sub(Cents::new(40).unwrap()), Cents::new(60));
        let low = Cents::new(-100_000_000).unwrap();
        assert!(low.checked_sub(Cents::new(1).unwrap()).is_none()); // leaves the partition

        assert_ne!(Balance::new(1).unwrap(), Balance::new(2).unwrap());
        assert!(Balance::new(1).unwrap() < Balance::new(2).unwrap());
        assert_eq!(
            Balance::new(2).unwrap().cmp(&Balance::new(2).unwrap()),
            core::cmp::Ordering::Equal
        );

        assert_eq!(format!("{:?}", Balance::new(5).unwrap()), "Qty(5)");
    }
}
