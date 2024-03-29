# ZFS Recordsize
#### 2019-06-11

At Joyent we operate an object storage system. One of the key problems in any
storage system is storing data as efficiently as possible. At cloud scale this
is more important than ever. Small percentage gains in storage capacity savings
have result in massive returns.

For the example, a 1% reduction in capacity needed to store one exabyte of data
saves 10 petabytes. TEN PETABYTES. If we have a storage server that has 250TB of
usable capacity, that's 40 storage servers worth of capacity we've saved. 40
boxes! That's absolutely crazy.

I was able to find a way for us to save between one and two percent capacity by
diving into the nitty gritty of the ZFS recordsize attribute.

ZFS isn't like other filesystems where block sizes are statically configured
when the system is initially deployed. Instead ZFS has what is called a
recordsize. A record is an atomic unit in ZFS. It's what is used to calculate
checksums, RAIDZ parity, and to perform compression.

The default recordsize is 128k, at least on illumos. What that means is that a
file you write will have a block size of _up to_ 128k in size. If you end up
writing a 64k file it will have a 64k block size. If you write a 256k file it
will be made up of two 128k records (and one indirect block to point to the two
128k records, but I digress).

RAIDZ parity is another concept we need to understand. ZFS allocates N disk
_sectors_ for each record written, where N is the parity level (0 thru 3). So
if we write an 8k file onto a RAIDZ1 pool with 4k sector size disks ZFS will
write one 8k record and one 4k RAIDZ parity sector. Padding sectors are
additionally added until the on-disk usage is a multiple of the parity level.
I'm not sure why this is done. In this example the RAIDZ storage overhead is
50%. We're using (about) 12k to store 8k of user data. That's pretty bad!

This is why a small recordsize is bad with RAIDZ. The efficiency is atrocious.

The larger the records, the less RAIDZ overhead, since RAIDZ overhead is mostly
constant per-record. Right? Maybe, but also maybe not. I thought that was going
to be the case initially, but after doing some math and observing how ZFS
behaves I am less certain.

We know what happens if we write _less_ than recordsized files, but what happens
when we write _more_ than recordsized files?

I wrote two files, and examined them with zdb(1m). Two filesystems were used.
One with a 128k recordsize and one with a 1M recordsize. 1M is the largest
recordsize currently without modifying the ZFS code (though ZFS supports
larger record sizes). These two files are larger than recordsize by only one
byte:

```
[root@coke /var/tmp/recordsize_testing]# ls -l /testpool/test1/ /testpool/test0
/testpool/test0:
total 532
-rw-r--r--   1 root     root      131073 Jun  5 20:45 worst_case

/testpool/test1/:
total 4091
-rw-r--r--   1 root     root     1048577 Jun  5 20:45 worst_case

[root@coke /var/tmp/recordsize_testing]# zdb -vvO testpool/test0 worst_case

    Object  lvl   iblk   dblk  dsize  dnsize  lsize   %full  type
         2    2   128K   128K   266K    512   256K  100.00  ZFS plain file
                                               168   bonus  System attributes
...
Indirect blocks:
               0 L1  0:8a1e00:1800 20000L/1000P F=2 B=214/214
               0  L0 0:cc00:27600 20000L/20000P F=1 B=214/214
           20000  L0 0:2a6600:27600 20000L/20000P F=1 B=214/214

        segment [0000000000000000, 0000000000040000) size  256K

[root@coke /var/tmp/recordsize_testing]# zdb -vvO testpool/test1 worst_case

    Object  lvl   iblk   dblk  dsize  dnsize  lsize   %full  type
         2    2   128K     1M  2.00M    512     2M  100.00  ZFS plain file
                                               168   bonus  System attributes
...
Indirect blocks:
               0 L1  0:8a3600:1800 20000L/1000P F=2 B=214/214
               0  L0 0:34200:139200 100000L/100000P F=1 B=214/214
          100000  L0 0:16d400:139200 100000L/100000P F=1 B=214/214

        segment [0000000000000000, 0000000000200000) size    2M
```

We can see that when we write more than recordsize, an _entire_ recordsized
record is allocated for the last record in an object. That means we have almost
100% overhead for these recordsize + 1 byte files.

This was a very unfortunate discovery, but I'm glad I noticed this before we
suggested deploying this recordsize change to production.

I ended up writing a pretty complicated calculator to simulate how ZFS would
use storage capacity. It's available
[here](https://github.com/KodyKantor/kodyops/blob/master/simulators/zfs_usage_simulator.sh).

It can take many arguments to tweak the simulator how you see fit, the most
important argument four our case was recordsize. We have the benefit of
our storage nodes uploading a manifest of files and file sizes that they store.
So I am able to quickly see how different recordsizes might lead to different
amounts of allocated storage based on real production data (a rare situation!).

This simulator gave us the knowledge we needed to determine the optimal
recordsize for the exact objects we are storing.

If you're _always_ storing objects slightly under 1M, a 1M recordsize will
definitely be most efficient in terms of space used for your data. In an object
storage system we have the benefit of the objects being immutable. This saves us
from many of the sticky points of using enormous recordsize values.

In our case we also store medium, but not large objects (where the
last-record overhead would be lost in the noise), so a 1M recordsize is not
best for us. The simulator confirmed this.

It outputs data like this:
```
Simulating using account *, parity 2, and recordsize 131072
=== REPORT ===
326233044036162         Bytes Used
4412350962088           Wasted Bytes
2035543840              Records
4071087680              RAIDZ sectors
7001110                 Padding sectors
296.71                  TiB Used
4                       TiB wasted
15.17                   TiB RAIDZ
```

Another thing to consider: the larger the recordsize the fewer records you will
have. This may help you avoid nasty fragmentation-related issues on your zpools.
