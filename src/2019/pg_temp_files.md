# Postgres Temporary Files
#### 2019-03-18

Today someone reported that our Postgres dashboard in Grafana was very choppy.
Prometheus is supposed to go out and scrape our pgstatsmon targets every minute.
There are three pgstatsmon targets in each region - one in each datacenter.

It turned out that the problem (as usual) was in Prometheus. Our Prometheus
instance was running out of memory and falling behind scraping targets.
However, since I was already logged into production I began looking at our
pgstatsmon instances to see if there were any problems with them.

Another person earlier mentioned that pgstatsmon was throwing occasional query
timeout and connection errors. pgstatsmon ships with a few simple DTrace scripts
just for situations like these.

In our first and second datacenters everything was clean. pgstatsmon had
connections to all 97 Postgres backends, and there were no operational errors
(query timeouts, connection errors) or programmer errors (query errors, NaN
errors). Our last datacenter was reporting an error connecting to a Postgres
backend. I logged into that backend's zone and saw that it was a deposed peer
(a former primary that had failed):

```
[root@aa7ab002 (postgres) ~]$ manatee-adm show
zookeeper:   zk_addr
cluster:     57.moray
generation:  8 (3DB0/AC7F6A78)
mode:        normal
freeze:      not frozen

ROLE     PEER     PG   REPL  SENT          FLUSH         REPLAY        LAG
primary  4dc75e3f ok   sync  3DB1/CBE280   3DB1/CBE0F0   3DB1/CB3668   -
sync     b3aa6bb1 ok   -     -             -             -             0m00s
deposed  aa7ab002 ok   -     -             -             -             -

warning: cluster has a deposed peer
warning: cluster has no async peers
```

So that wasn't pgstatsmon's fault, but something that we should investigate
later. This explains the latent connection errors that were reported.

While looking into this I had left the other pgstatsmon DTrace scripts running.
In the intervening time the other pgstatsmon instances reported a number of
query timeouts to a few shards. Digging deeper with another DTrace script, this
is what we see:

```
36.postgres.my.domain-090930c5
               QUERY      LAT QTIM QERR  NaN
 pg_stat_user_tables      469    0    0    0
pg_statio_user_tables     487    0    0    0
pg_statio_user_indexes    496    0    0    0
 pg_stat_replication      509    0    0    0
         pg_recovery      511    0    0    0
    pg_stat_activity      517    0    0    0
    pg_stat_database     1001    1    0    0
    pg_relation_size     1002    1    0    0
    pg_stat_bgwriter     1002    1    0    0
pg_stat_progress_vacuum  1003    1    0    0
           pg_vacuum     1003    1    0    0
```

The columns are:
- QUERY: the 'name' of the query. Usually this refers to the primary
		data sourcewhere pgstatsmon gets its data.
- LAT: cumulative latency for queries to this backend.
- QTIM: 1 if the query timed out.
- QERR: 1 if an error was returned from Postgres.
- NaN: 1 if the data returned was a NaN type in Javascript.

Right off the bat, these queries should all finish in less than 300ms. The first
query usually takes about 20ms. pgstatsmon timed out the queries after they took
1s of cumulative time. But why were these queries taking so long? I logged in to
the Postgres instance to investigate.

The first thing I looked at in the Postgres zone was its log file. It didn't
take long to find a potential problem.

```
2019-03-18 19:32:45 UTC LOG:  temporary file: path "base/pgsql_tmp/pgsql_tmp85092.468", size 1018377378
2019-03-18 19:32:45 UTC STATEMENT:  SELECT *, '916f7bc4-8e55-647c-8a16-96a48c4895ec' AS req_id FROM manta_fastdelete_queue WHERE  ( $1 <= _mtime AND _mtime IS NOT NULL )  LIMIT 3500 OFFSET 7000
```

manta_fastdelete_queue is a Postgres relation that we use to store information
about files ready for deletion. This is part of relatively new 'accelerated GC'
code in Manta. The accelerated GC code is the only code that should be touching
this table, and it is not expected that queries should be creating temporary
files.

Next I looked at the temp files on disk to see how many and how large they were:

```
[root@090930c5 (postgres) ~]$ ls -lh /manatee/pg/data/base/pgsql_tmp/
total 6.1G
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85093.501
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85093.502
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85093.503
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85093.504
-rw------- 1 postgres root 1.0G Mar 18 19:52 pgsql_tmp85093.505
-rw------- 1 postgres root 972M Mar 18 19:52 pgsql_tmp85093.506
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85161.506
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85161.507
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85161.508
-rw------- 1 postgres root 1.0G Mar 18 19:51 pgsql_tmp85161.509
-rw------- 1 postgres root 1.0G Mar 18 19:52 pgsql_tmp85161.510
-rw------- 1 postgres root 972M Mar 18 19:52 pgsql_tmp85161.511
```

That's unfortunate. An interesting observation - 'ls' reports 6.1G at the top
level, but there are about 12 1GB files in the listing... I also
verified that these queries were showing up in pg_stat_activity.

The Postgres docs on LIMIT and OFFSET note that the OFFSET has to be computed by
the server and may cause performance problems. Looking at the EXPLAIN of the
query being used gives us some answers:

```
moray=> explain SELECT * FROM manta_fastdelete_queue WHERE  ( 1000 <= _mtime AND _mtime IS NOT NULL )  LIMIT 3500 OFFSET 3500;
                                        QUERY PLAN
-------------------------------------------------------------------------------------------
 Limit  (cost=1210.85..2421.70 rows=3500 width=1410)
   ->  Seq Scan on manta_fastdelete_queue  (cost=0.00..1551769.54 rows=4485435 width=1410)
         Filter: ((_mtime IS NOT NULL) AND (1000 <= _mtime))
(3 rows)
```

That tells us that this query is most likely going to try to scan the entire
manta_fastdelete_queue table. This is probably why we're hitting work_mem and
making temporary files.

It also begs another question. Why didn't it list anything about `OFFSET` or
`LIMIT` in the output?

Based on what I've seen my theory is that the `OFFSET` directive is causing the
backend process to buffer much of the table in memory to compute the `OFFSET`.
Our work_mem is set to a measly 3MB (which has never led to this problem in the
past) and this relation on disk is about 12GB:

```
[root@090930c5 (postgres) ~]$ ls -lh /manatee/pg/data/base/16385/74462*
-rw------- 1 postgres root 1.0G Mar 18 20:40 /manatee/pg/data/base/16385/74462
-rw------- 1 postgres root 1.0G Mar 18 20:43 /manatee/pg/data/base/16385/74462.1
-rw------- 1 postgres root 1.0G Mar 18 19:41 /manatee/pg/data/base/16385/74462.10
-rw------- 1 postgres root 422M Mar 18 19:50 /manatee/pg/data/base/16385/74462.11
-rw------- 1 postgres root 1.0G Mar 18 20:53 /manatee/pg/data/base/16385/74462.2
-rw------- 1 postgres root 1.0G Mar 18 21:03 /manatee/pg/data/base/16385/74462.3
-rw------- 1 postgres root 1.0G Mar 18 20:50 /manatee/pg/data/base/16385/74462.4
-rw------- 1 postgres root 1.0G Mar 18 21:10 /manatee/pg/data/base/16385/74462.5
-rw------- 1 postgres root 1.0G Mar 18 21:10 /manatee/pg/data/base/16385/74462.6
-rw------- 1 postgres root 1.0G Mar 18 20:09 /manatee/pg/data/base/16385/74462.7
-rw------- 1 postgres root 1.0G Mar 18 20:32 /manatee/pg/data/base/16385/74462.8
-rw------- 1 postgres root 1.0G Mar 18 19:39 /manatee/pg/data/base/16385/74462.9
```

### June Update

It appears that I was on to something deeper here. I was looking at another
system during a recent trip to Korea and noticed that there were some queries
blocking on the WALWriteLock. The WALWriteLock is infamous for being on the
scene during Postgres performance issues. It needs to be acquired whenever a
record is inserted into the WAL. IIUC this happens whenever a transaction
modifies table data.

I took a [statemap](https://github.com/joyent/statemap) of the system I was
looking at. These are the things I observed:
- Multiple processes blocking on locks (presumably WALWriteLock)
- A few processes spending _way_ too much time in zil_commit

I then used DTrace to track zil_commit latencies, and the results were damning.
Some zil_commits were taking over 200ms! Since zil_commit is how ZFS implements
fsync it's no wonder things were performing pathologically.

My coworker Jerry was able to pretty quickly determine that the zil_commit ZIOs
were getting delayed in the ZIO pipeline, which was causing much of the latency.

I also wrote a [complicated DTrace](https://github.com/KodyKantor/kodyops/blob/master/simulators/zfs_usage_simulator.sh) script (and found a DTrace bug on the way!)
to track where ZIOs are spending time in the ZIO pipeline.
It's a riff on an 'extended' DTrace script that George Wilson presented at
the 2018 OpenZFS Summit. My version is a little more complicated, since it only
prints pipelines that are over a given time threshold (in your time unit of
choice) and also calculates time the ZIO spent waiting (not being executed).

The result of running my zio.d script is this:

```
	[65532ns] zil_lwb_write_issue
	[20317ns] zio_write_bp_init
	[27606ns] wait
	[15031ns] zio_issue_async
	[16879ns] wait
	[11901ns] zio_write_compress
	[11568ns] wait
	[13422ns] zio_checksum_generate
	[12041ns] wait
	[10437ns] zio_ready
	[11355ns] wait
	[27992ns] zio_vdev_io_start
	[25709ns] wait
	[9557ns] zio_vdev_io_done
	[314426ns] wait
	[13820ns] zio_vdev_io_done
	[8677ns] wait
	[18070ns] zio_vdev_io_assess
	[17351ns] wait
	[81714ns] zio_done
	[10576ns] wait
	[743981ns] DTrace calculated duration
	[664521ns] ZIO reported duration
```

Pretty useful!

In the end we discovered that the ZFS bug we were encountering had been fixed
a few months ago (illumos#9993 fixed in Nov 2018), which was caused by commit
illumos#19097 - "zfs i/o scheduler needs some work."