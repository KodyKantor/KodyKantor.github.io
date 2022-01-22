# pg_xlogdump(1) - a tale of 15 round trips
#### 2018-11-12

In the last year I've been spending more time using DTrace, MDB, and other tools
to observe and debug systems. I thought I'd write down some of the interesting
things I've written or discovered.


## Exploring Postgres

`pg_xlogdump` (`pg_waldump` for those of us in the PG 10+ future - not me) is a
really powerful tool that dumps out the contents of the Postgres write ahead log
files. This can be used for all sorts of data wrangling, although from my
limited time with the tool I didn't see any directly machine-parseable output.

pg_xlogdump was
[added](https://paquier.xyz/postgresql-2/postgres-9-3-feature-highlight-pg_xlogdump/)
in PG 9.3 as a 'contrib' add-on. Before this there were a few things floating
around that had similar functionality but they were maintained outside of
Postgres.

Renaming pg_xlogdump to pg_waldump was done with a slew of other renames in PG
10. Why would that be? Because people were deleting directories in the PG
portion of their filesystem that had the word 'log' in them (why is this log
stuff taking all of my IOPS?!?! - famous last words of a frustrated sysadmin).
Some background on these changes are here:
[original post](https://www.postgresql.org/message-id/flat/CAASwCXcVGma9KgEu-ESC6u928mW67noZvnawbPUSW7R7AN9UVg%40mail.gmail.com),
[implementation discussion](https://www.postgresql.org/message-id/flat/CAB7nPqTeC-8%2Bzux8_-4ZD46V7YPwooeFxgndfsq5Rg8ibLVm1A%40mail.gmail.com).
Here's a [summary](https://wiki.postgresql.org/wiki/New_in_postgres_10#Renaming_of_.22xlog.22_to_.22wal.22_Globally_.28and_location.2Flsn.29) of changes.

So the skinny of it is that people were deleting anything in the Postgres data
directories that had the word 'log' in them. This was probably an uninformed
decision on the part of the operator(s), but Michael Paquier did point out that
the original names of things weren't very indicative of what their function was
(e.g. pg_xlog isn't as self-explanatory as pg_wal).

Anyway, a tool like pg_xlogdump is most interesting with an example.
The pg_xlogdump tool can be given the name of a WAL file and it will dump
somewhat human-readable output to the console. That's a little verbose for us,
so we'll look at a single transaction. This is an INSERT into the 'manta' table
of a development deployment of Joyent's Manta service:


```
[root@cf90e54c (postgres) /manatee/pg/data/pg_xlog]$ pg_xlogdump 00000001000004210000007B -x 1592894815
rmgr: Heap        len (rec/tot):      3/   751, tx: 1592894815, lsn: 421/7BFF9B60, prev 421/7BFF9A98, desc: INSERT off 1, blkref #0: rel 1663/16385/18650 blk 8455231
rmgr: Btree       len (rec/tot):      2/   160, tx: 1592894815, lsn: 421/7BFFA1F8, prev 421/7BFF9EF0, desc: INSERT_LEAF off 28, blkref #0: rel 1663/16385/57453 blk 16791519
rmgr: Btree       len (rec/tot):      2/    96, tx: 1592894815, lsn: 421/7BFFE120, prev 421/7BFFE0D8, desc: INSERT_LEAF off 56, blkref #0: rel 1663/16385/57454 blk 5182563
rmgr: Btree       len (rec/tot):      2/    96, tx: 1592894815, lsn: 421/7BFFE180, prev 421/7BFFE120, desc: INSERT_LEAF off 2, blkref #0: rel 1663/16385/57455 blk 8753025
rmgr: Btree       len (rec/tot):      2/    72, tx: 1592894815, lsn: 421/7BFFE1E0, prev 421/7BFFE180, desc: INSERT_LEAF off 2, blkref #0: rel 1663/16385/57457 blk 3984299
rmgr: Btree       len (rec/tot):      2/    72, tx: 1592894815, lsn: 421/7BFFE330, prev 421/7BFFE2B0, desc: INSERT_LEAF off 202, blkref #0: rel 1663/16385/57459 blk 1466454
rmgr: Btree       len (rec/tot):      2/    64, tx: 1592894815, lsn: 421/7BFFE378, prev 421/7BFFE330, desc: INSERT_LEAF off 81, blkref #0: rel 1663/16385/57460 blk 3291661
rmgr: Btree       len (rec/tot):      2/   128, tx: 1592894815, lsn: 421/7BFFE3F8, prev 421/7BFFE3B8, desc: INSERT_LEAF off 59, blkref #0: rel 1663/16385/57462 blk 13931439
rmgr: Btree       len (rec/tot):      2/    64, tx: 1592894815, lsn: 421/7BFFE478, prev 421/7BFFE3F8, desc: INSERT_LEAF off 241, blkref #0: rel 1663/16385/57463 blk 230841
rmgr: Transaction len (rec/tot):     20/    46, tx: 1592894815, lsn: 421/7BFFFA08, prev 421/7BFFF9C8, desc: COMMIT 2018-11-13 03:19:43.829708 UTC; subxacts: 1592894828

[root@cf90e54c (postgres) /manatee/pg/data/pg_xlog]$ pg_xlogdump 00000001000004210000007B -x 1592894828
rmgr: Heap        len (rec/tot):      3/   164, tx: 1592894828, lsn: 421/7BFFE4B8, prev 421/7BFFE478, desc: INSERT off 46, blkref #0: rel 1663/16385/18641 blk 10606796
rmgr: Btree       len (rec/tot):      2/   128, tx: 1592894828, lsn: 421/7BFFE560, prev 421/7BFFE4B8, desc: INSERT_LEAF off 31, blkref #0: rel 1663/16385/18643 blk 12515860
rmgr: Btree       len (rec/tot):      2/    64, tx: 1592894828, lsn: 421/7BFFE5E0, prev 421/7BFFE560, desc: INSERT_LEAF off 196, blkref #0: rel 1663/16385/90112 blk 1614613
rmgr: Btree       len (rec/tot):      2/    72, tx: 1592894828, lsn: 421/7BFFE620, prev 421/7BFFE5E0, desc: INSERT_LEAF off 2, blkref #0: rel 1663/16385/98305 blk 1
rmgr: Btree       len (rec/tot):      2/    64, tx: 1592894828, lsn: 421/7BFFE668, prev 421/7BFFE620, desc: INSERT_LEAF off 292, blkref #0: rel 1663/16385/98304 blk 1614518
```
This command looked at the xlog
`00000001000004210000007B` for XID `1592894815`. The first line (`Heap`) denotes
the data being recorded. The next eight lines are all index data being modified.
The final line is the transaction being committed. Note that it references
a `subxact`, which I also pasted.

The `len (rec/tot)` section I take to represent the total length of data being
recorded. I'm not sure what the `rec` field means. Then we can see the
transaction number (what we queried on), and a pair of LSNs. The LSN type was
first-classed in PG 9.4 and is a monotonically incrementing number behind the
scenes. The `lsn` in the above output is the logical sequence number assigned
to each of the WAL segments. The `prev` LSN is the LSN of the previously written
WAL segment. I believe this is used in the case that the WAL has to be replayed.
So this facility would ensure that the segments are replayed in sequence in the
case that entries in the file are moved around.

The `desc` section denotes what type of operation happened. INSERT_LEAF means
data needs to be added to the index's btree for example. The next interesting
bit that I somewhat understand is `rel` which explains what's being modified.

Take the first line of the first dump:

```
rel 1663/16385/18650
```
Here 1663 represents the 'base' tablespace, 16385 is the 'moray' database, and
18650 is the 'manta' relation.

I started from the bottom:


```
moray=# select relname from pg_class where relfilenode = 18650;
 relname
---------
 manta
(1 row)

moray=# select datname from pg_database where oid = 16385;
 datname
---------
 moray
(1 row)

moray=# select pg_relation_filepath('manta');
 pg_relation_filepath
----------------------
 base/16385/18650
(1 row)
```
And then we can do the same for the subtransaction:


```
moray=# select relname from pg_class where relfilenode = 18641;
        relname
------------------------
 manta_directory_counts
(1 row)
```

Now we know that this series of WAL records represents an INSERT into the
'manta' table, how much data was written, in what order, and that it (correctly)
kicked off a trigger to update manta_directory_counts.

I should also note that pg_xlogdump will indicate if a write was a full-page
write, which is important for some performance investigations.

This was just one small example of what we can learn from the PG WAL using
pg_xlogdump. You could conceivably graph a lot of this data over time and draw
interesting conclusions from it.

The takeaways here:
- Every individual data modification uses an LSN, grabs a WAL writer lock, and
	  results in a round-trip to the synchronous replica (with varying levels of
	  consistency based on how you configured your database).
- One logical insert into the 'manta' table resulted in 15 round trips to
	  the PG replica and 2041 (logical) bytes of data written to WAL, 1080 of which
	  was index data.
- There's a ton of data in the WAL.
- Understanding the WAL can be a great asset when developing new PG schemas.

The existing 'manta' schema isn't the best when it comes to improving latency
and throughput, and minimizing storage usage. If the schema had more
heap-only-tuple updates (HOT updates), the latency of inserts would be greatly
reduced. Note that the manta_directory_counts relation sometimes has HOT updates
due to the limited number of indexes it maintains.

I hope this was interesting! It was nice for me to go back and record this
despite not having touched pg_xlogdump (or my blog!) for many months now.