# finql

Purpose of this library is to provide over time a comprehensive toolbox
for quantitative analysis of financial assets in rust. 
The project is licensed under Apache 2.0 or MIT license (see files LICENSE-Apache2.0 and LICENSE-MIT)
at the option of the user.

The goal is to provide a toolbox for analyzing various financial products, like stocks, bonds,
and sometime maybe even more complex or derivative products. 
In the near term, calculation of the discounted cash flow value of bonds
is in the focus, based on what information is given by a standard prospect. Building blocks to achieve
this target include time periods (e.g. "3M" or "10Y"), bank holiday calendars, business day adjustment
rules, calculation of year fraction with respect to typical day count convention methods, roll-out of
cash flows, and calculating valuation and risk figures like internal yield or duration that are useful 
for an investor in these products. 

Functionality to calculate of figures like fair values which are primarily interesting in scenarios 
where one is fully hedged are not in the initial focus, since an investor is by definition not
fully hedged. Nevertheless, they might be added later for comparison and estimating market prices.

