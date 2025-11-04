use std::io;

#[allow(clippy::redundant_closure)]
#[must_use]
pub fn read_index() -> usize {
    let mut input = String::new();
    loop {
        if io::stdin().read_line(&mut input).is_ok()
            && let Ok(output) = input.trim().parse::<usize>()
        {
            return output;
        }
    }
}

#[must_use]
pub fn binomial(mut n: usize, mut k: usize) -> usize {
    if k > n {
        return 0;
    }
    if k > (n / 2) {
        k = n - k;
    }
    let mut result = 1;
    for d in 1..=k {
        result *= n;
        n -= 1;
        result /= d;
    }
    result
}

#[test]
fn helpers_tests() {
    assert_eq!(binomial(24, 6), 134_596);
}
