/// Number of ways to choose `k` items among `n`, useful to weigh a hand.
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

#[cfg(test)]
mod tests {
    use super::binomial;

    #[test]
    fn binomial_counts_combinations() {
        assert_eq!(binomial(24, 6), 134_596);
        assert_eq!(binomial(5, 0), 1);
        assert_eq!(binomial(3, 5), 0);
    }
}
