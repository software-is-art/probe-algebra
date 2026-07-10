impl ::boundary_spec::discover::lift::Liftable for Tally {
    fn theory_name() -> &'static str { "tally" }
    fn ops() -> ::std::vec::Vec<::boundary_spec::discover::lift::LiftedOp<Self>> {
        ::std::vec![
            ::boundary_spec::discover::lift::LiftedOp { name: "merge", symbol: "merge", fixity: ::boundary_spec::discover::engine::Fixity::Infix, arity: 2, eval: |a| ::std::option::Option::Some(merge(a[0].clone(), a[1].clone())) },
            ::boundary_spec::discover::lift::LiftedOp { name: "floor", symbol: "floor", fixity: ::boundary_spec::discover::engine::Fixity::Nullary, arity: 0, eval: |_| ::std::option::Option::Some(floor()) },
            ::boundary_spec::discover::lift::LiftedOp { name: "bump", symbol: "bump", fixity: ::boundary_spec::discover::engine::Fixity::Prefix, arity: 1, eval: |a| ::std::option::Option::Some(bump(a[0].clone())) },
        ]
    }
    fn expectations() -> ::std::vec::Vec<::boundary_spec::discover::expect::Expectation> {
        ::std::vec![
            ::boundary_spec::discover::expect::Expectation::of("commutativity", ::std::vec!["merge"]),
            ::boundary_spec::discover::expect::Expectation::of("associativity", ::std::vec!["merge"]),
            ::boundary_spec::discover::expect::Expectation::of("idempotence", ::std::vec!["merge"]),
            ::boundary_spec::discover::expect::Expectation::of("identity", ::std::vec!["merge", "floor"]),
        ]
    }
}
