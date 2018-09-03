use rule::*;

// TODO: First, we need liftetimed graphs.
// Second, we need to actually build the combinator.
// Undecided if we should test that it's as expected.
lazy_static! {
    static ref plus_term: Term = Term::new('+' as u8);
    static ref mul_term: Term = Term::new('*' as u8);
    static ref two_term: Term = Term::new('2' as u8); // ascii '2'

    static ref mononomial_thunk: Thunk = Thunk::new();
    static ref mononomial_complex_slice: Vec<&'static Rule> =
        vec!(&*two_term, &*mul_term, &*mononomial_thunk);
    static ref mononomial_complex: Seq = Seq::new(&mononomial_complex_slice);
    static ref mononomial_slice: Vec<&'static Rule> = vec!(&*two_term, &*mononomial_complex);
    static ref mononomial: Choice = Choice::new(&mononomial_slice);
    static ref mononomial_init: () = {
        mononomial_thunk.set(&*mononomial);
    };

    static ref polynomial_thunk: Thunk = Thunk::new();
    static ref polynomial_complex_slice: Vec<&'static Rule> =
        vec!(&*two_term, &*plus_term, &*polynomial_thunk);
    static ref polynomial_complex: Seq = Seq::new(&polynomial_complex_slice);
    static ref polynomial_slice: Vec<&'static Rule> = vec!(&*mononomial, &*polynomial_complex);
    static ref polynomial: Choice = Choice::new(&polynomial_slice);
    static ref polynomial_init: () = {
        polynomial_thunk.set(&*polynomial);
    };

    static ref head: Choice = {
        *polynomial_init; // force init
        *polynomial
    };
}

#[test]
#[allow(unused_must_use)]
fn smoke_test() {
    *head;
}
