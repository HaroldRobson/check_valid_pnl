use std::collections::HashMap;
use std::collections::HashSet;
/*
* so stock changes are day to day in percent e,g. [-7, 3, 12]
* portfolio claims are of the form [1911, 733, -222] in basis points
* holding period H is in days
* return a vec of tuples corresponding to the start and end date of purchases which would replicate
* the claimed basis point change
* if no such strategy can be made, return (-1, -1)
*/

/*
My Assumptions:
1) Stock change is relative to the previous days value. For example [-7, 3] represents a stock change of 0.93*1.03 != 1
2) A portfolio can either contain one stock, or none. It cannot for example buy 1 stock, then 3 days later buy another 2
3) A Portfolio needn't merely buy once and sell once. It could, for example, buy a stock at time t, sell at t+H, buy at t+H+1, and sell at t+2H+1.
4) The start and end times for a given claim are not unique.
5) A sale occurs before the price change for that day. For example, if the stock_changes are [-3, -2, -1], and the (start, end) is (0, 2),
    then the portfolio_claim is (0.97*0.98)*10_000, NOT (0.97*0.98*0.99)*10_000
6) The claim for a given start and end time is not unique. For example, if H were 2, then the interval (0, 6) could have the same value as (0, 2)*(4, 6), or (0, 2)*(3, 6)
*/
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
struct Claim {
    delta: i32, // in bsp
    start: i32,
    end: i32,
}
impl Claim {
    fn new(delta: i32, start: i32, end: i32) -> Self {
        Self { delta, start, end }
    }
}

fn check_values(stock_changes: Vec<i32>, portfolio_claims: Vec<i32>, H: i32) -> Vec<(i32, i32)> {
    // change the portfolio claims to be the multiplicative change, not the additive
    let portfolio_claims: Vec<i32> = portfolio_claims.iter().map(|x| x + 10_000).collect();

    let n = stock_changes.len();
    let h = H as usize;
    let mut sell_before_possibilities: Vec<HashSet<Claim>> = vec![HashSet::new(); n + 2];
    let mut valid_claims: HashMap<i32, Claim> = HashMap::new();
    for i in h..n + 1 {
        for j in 0..=i - h {
            let mut mult = 1.00;
            for k in j..i {
                mult *= (100 + stock_changes[k]) as f64 / 100.0;
            }
            let new_delta = (mult * 1000_000.0) as i32; // we need 10^6 accuracy to avoid
                                                        // flointing point errors. if these were just 10^4, then we could have eg 10835 and
                                                        // 10836 as both valid values for the same range.
            let new_claim = Claim::new(new_delta, j as i32, i as i32);
            sell_before_possibilities[i].insert(new_claim);
            valid_claims.insert(new_delta, new_claim);
            for s in sell_before_possibilities[j].clone() {
                if s.end > j as i32 {
                    continue;
                }
                let new_delta = (s.delta as f64 * mult).round() as i32;
                let new_claim = Claim::new(new_delta, s.start, i as i32);
                sell_before_possibilities[i].insert(new_claim);
                valid_claims.insert(new_delta, new_claim);
            }
        }
        let sell_before_i = sell_before_possibilities[i].clone();
        sell_before_possibilities[i + 1].extend(&sell_before_i);
    }
    let mut valid_claims_scaled: HashMap<i32, (i32, i32)> = HashMap::new();
    for claim in &valid_claims {
        let claim = claim.1;
        let delta_bps = (claim.delta as f64 / 100.0).round() as i32;
        valid_claims_scaled.insert(delta_bps, (claim.start, claim.end));
    }
    //dbg!(&sell_before_possibilities);
    //dbg![&valid_claims];
    //dbg![&valid_claims_scaled];
    portfolio_claims
        .iter()
        .map(|x| *valid_claims_scaled.get(x).unwrap_or(&(-1, -1)))
        .collect()
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complicated_multi_strategy() {
        let stock = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        /*
        lets say the PM bought at 0, sold at 3, bought at 6, sold at 8, bought at 11, sold at 14
        [ [(1.01*1.02*1.03) * (1.07*1.08) * (1.12*1.13*1.14) ] -1 ] * 10_000 = 7692
        */

        /*
        for the second strategy, lets say they bought at 2, sold at 10, bought at 13, sold at 15
        [ [(1.03*...1.10) * (1.14*1.15)] -1 ] * 10_000 = 11656.84 ~= 11657. Note that 11656 is invalid.
        */
        let claims = vec![7692, 11657, 11656];
        let h = 2;
        let result = check_values(stock, claims, h);
        assert_eq!(result[0], (0, 14));
        assert_eq!(result[1], (2, 15));
        assert_eq!(result[2], (-1, -1)); // to check decimals.
    }

    #[test]
    fn test_many_repurchases() {
        let stock = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ];
        /*
        now lets say they bought at 3, sold at 7, bought at 8, sold at 12, bought at 14, sold at 19
        [[(1.04*1.05*1.06*1.07)*(1.09*1.1*1.11*1.12)*(1.15*1.16*1.17*1.18*1.19)] -1 ] * 10_000 = 30461.6 ~= 30462
        */
        let claims = vec![30462, 30461];
        let h = 4;
        let result = check_values(stock, claims, h);
        assert_eq!(result[0], (3, 19));
        assert_eq!(result[1], (-1, -1));
    }

    #[test]
    fn test_holding_period_long() {
        let stock = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 100, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ];
        /*
        now lets say they bought at 3, sold at 10, bought at 12, sold at 20
        [[(1.04*1.05*1.06*1.07 * 1.08 * 1.09 * 1.10)*(1.12*1.13*1.14*1.15*1.16*1.17*1.18*1.19*1.20)] -1 ] * 10_000 = 50856.7 ~= 50857
        this would require a holding period of 8, but only 7 is provided:
        */
        let claims = vec![50857, 50856];
        let h = 8;
        let result = check_values(stock.clone(), claims.clone(), h);
        assert_eq!(result[0], (-1, -1));
        assert_eq!(result[1], (-1, -1));

        // but this should be possible with a holding period of 7:
        let h = 7;
        let result = check_values(stock.clone(), claims.clone(), h);
        assert_eq!(result[0], (3, 20));
        assert_eq!(result[1], (-1, -1)); // checking decimals again
    }

    #[test]
    fn test_volatile_precision() {
        let mut stock = vec![-50; 20];
        for i in 1..10 {
            stock[2 * i - 1] = 50;
        }
        // [(1.5^7 * 0.5^6) - 1] * 10_000 = -7330.32226563 ~= -7330
        let h = 2;
        let claims = vec![-7330, -7331, -7329];
        let result = check_values(stock.clone(), claims.clone(), h);
        assert_ne!(result[0], (-1, -1));
        assert_eq!(result[1], (-1, -1)); // checking decimals again
        assert_eq!(result[2], (-1, -1)); // checking decimals again

        //may as well check the holding period
        let h = 14;
        let result = check_values(stock.clone(), claims.clone(), h);
        assert_eq!(result[0], (-1, -1));
    }

    #[test]
    fn test_basic_single_trade() {
        // 10% gain followed by 10% gain
        // (1.10 * 1.10) = 1.21. Total gain = 21% = 2100 bps
        let stock = vec![10, 10];
        let claims = vec![2100];
        let h = 2;
        let result = check_values(stock, claims, h);
        assert_eq!(result, vec![(0, 2)]); // Assumption 5: end is index + 1
    }

    #[test]
    fn test_minimum_holding_period() {
        let stock = vec![5, 5, 6];
        let claims = vec![1025, 1130]; // (1.05 * 1.05), (1.05*1.06)
        let h = 3;
        let result = check_values(stock.clone(), claims.clone(), h);
        assert_eq!(result[0], (-1, -1));
        assert_eq!(result[1], (-1, -1));
        //try again with 2
        let h = 2;
        let result = check_values(stock.clone(), claims.clone(), h);
        // A 1-day trade would be 500 bps, but H=2, so it should fail
        assert_eq!(result[0], (0, 2));
        assert_eq!(result[1], (1, 3));
    }

    #[test]
    fn test_negative_claims() {
        let stock = vec![1, 2, -5, -5, 0, 0, 4];
        let claims = vec![-975];
        let h = 4;
        let result = check_values(stock, claims, h);
        assert_eq!(result, vec![(2, 6)]);
    }

    #[test]
    fn test_precision_and_rounding() {
        let stock = vec![3; 6];
        let claims = vec![1941, 1940];
        let h = 1;
        let result = check_values(stock, claims, h);
        assert_eq!(result, vec![(0, 6), (-1, -1)]);
    }

    #[test]
    fn test_impossible_claim() {
        let stock = vec![1, 2, 3];
        let claims = vec![9999]; // Impossible gain
        let h = 1;
        let result = check_values(stock, claims, h);
        assert_eq!(result, vec![(-1, -1)]);
    }

    #[test]
    fn test_multiple_claims() {
        let stock = vec![10, -10, 20];
        let claims = vec![1000, -100];
        let h = 1;
        let result = check_values(stock, claims, h);
        assert_eq!(result, vec![(0, 1), (0, 2)]);
    }

    #[test]
    fn test_h_equals_one() {
        // With H=1, any single day is a valid trade.
        // stock: [10, -10]
        // Claims: 1000 (Day 0-1), -1000 (Day 1-2), -100 (Day 0-2)
        let stock = vec![10, -10];
        let h = 1;

        let results = check_values(stock, vec![1000, -1000, -100], h);
        assert_eq!(results[0], (0, 1)); // Single day 0
        assert_eq!(results[1], (1, 2)); // Single day 1
        assert_eq!(results[2], (0, 2)); // Both days compounded
    }

    #[test]
    fn test_h_equals_total_length() {
        let stock = vec![10, 10, 10];
        let h = 3;
        let claim_ok = vec![3310];
        let claim_fail = vec![1000, 2100]; // Individual days or 2-day windows

        assert_eq!(check_values(stock.clone(), claim_ok, h), vec![(0, 3)]);
        assert_eq!(check_values(stock, claim_fail, h), vec![(-1, -1), (-1, -1)]);
    }

    #[test]
    fn test_multi_trade_reinvestment() {
        let stock = vec![10, -50, 10, 10];
        let h = 1;
        let claims = vec![3310];

        let result = check_values(stock, claims, h);

        assert_eq!(result[0], (0, 4)); // skipping day 2 - allowed because h = 1
    }

    #[test]
    fn test_long_vector_with_gap() {
        let mut stock = vec![2; 6];
        stock[2] = -9;
        stock[5] = 9;

        let h = 2;
        let claim = vec![824, -150, -149]; // the -150

        let result = check_values(stock, claim, h);
        assert_eq!(result[0], (0, 5)); // so it bought at 0, sold at 2 (before crash), bought at 3,
                                       // sold at 5
        assert_eq!(result[1], (0, 5)); // bought at 0, sold at 5
        assert_eq!(result[2], (-1, -1)); // this would be (0, 5) if we didnt have the 10^6 scaling,
                                         // as floating point errors would introduce -149 as a valid key.
    }

    #[test]
    fn test_zero_change_days_long() {
        let mut stock = vec![0; 40];
        stock[3] = 2;
        stock[39] = 6;
        let h = 5;
        let claim = vec![812];
        let result = check_values(stock, claim, h);
        assert_eq!(result[0], (3, 40));
    }

    #[test]
    fn test_zero_change_days() {
        // Testing that "Sitting in cash" or 0% days don't break the multiplier
        let stock = vec![1, 0, 0, 10, 0, 8];
        let h = 4;
        let claims = vec![1000];
        let result = check_values(stock, claims, h);
        assert_eq!(result[0], (1, 5));
    }
}
