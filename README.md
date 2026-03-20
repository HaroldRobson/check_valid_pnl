# check_valid_pnl

run with "cargo test" to see the 16 tests implemented

The function check_values  takes in:
 - a vector of claimed portfolio performances in basis points, eg 800 means a portfolio lost 8%
 - a vector of day stock changes in percent
 - a holding period h

It assumes that:
 - Once purchased, stock must be held for h days before selling
 - Stock may be rebought after selling
 - At any time, a portfolio can either hold 1 stock, or hold no stocks. 
 - (Risk Free Rate is 0, time is discrete over days etc)

