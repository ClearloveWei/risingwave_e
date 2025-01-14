-- noinspection SqlNoDataSourceInspectionForFile
-- noinspection SqlResolveForFile
CREATE SINK nexmark_q16 AS
SELECT channel,
       to_char(date_time, 'YYYY-MM-DD')                                          as "day",
       max(to_char(date_time, 'HH:mm'))                                          as "minute",
       count(*)                                                                  AS total_bids,
       count(*) filter (where price < 10000)                                     AS rank1_bids,
       count(*) filter (where price >= 10000 and price < 1000000)                AS rank2_bids,
       count(*) filter (where price >= 1000000)                                  AS rank3_bids,
       count(distinct bidder)                                                    AS total_bidders,
       count(distinct bidder) filter (where price < 10000)                       AS rank1_bidders,
       count(distinct bidder) filter (where price >= 10000 and price < 1000000)  AS rank2_bidders,
       count(distinct bidder) filter (where price >= 1000000)                    AS rank3_bidders,
       count(distinct auction)                                                   AS total_auctions,
       count(distinct auction) filter (where price < 10000)                      AS rank1_auctions,
       count(distinct auction) filter (where price >= 10000 and price < 1000000) AS rank2_auctions,
       count(distinct auction) filter (where price >= 1000000)                   AS rank3_auctions
FROM bid
GROUP BY channel, to_char(date_time, 'YYYY-MM-DD')
WITH ( connector = 'blackhole', type = 'append-only', force_append_only = 'true');
