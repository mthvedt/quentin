use rule::*;

fn calculator() {
    let plus_term = Term::new('+' as u8);
    let mul_term = Term::new('*' as u8);
    let two_term = Term::new('2' as u8);

    let mononomial_complex_slice: Vec<&AsRuleKey> = vec![&two_term, &mul_term, &"mononomial"];
    let mononomial_complex = Seq::new(&mononomial_complex_slice);
    let mononomial_slice: Vec<&AsRuleKey> = vec![&two_term, &mononomial_complex];
    let mononomial = Choice::new(&mononomial_slice);

    let polynomial_complex_slice: Vec<&AsRuleKey> = vec![&two_term, &plus_term, &"polynomial"];
    let polynomial_complex = Seq::new(&polynomial_complex_slice);
    let polynomial_slice: Vec<&AsRuleKey> = vec![&mononomial, &polynomial_complex];
    let polynomial = Choice::new(&polynomial_slice);

    let mut g = Grammar::new();
    g.put("mononomial", &mononomial);
    g.put("polynomial", &polynomial);

    let alloc = ItemSetAllocator::new();

    build_items(&polynomial, &g, &alloc);
}

#[test]
fn smoke_test() {
    calculator();
}
