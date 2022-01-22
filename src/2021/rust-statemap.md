# Statemap Library for Rust Programs
#### 2021-04-14

I suppose this blog is turning into a place where I can talk about my obsession
with writing simulators. My hope is that if I dump my thoughts here where
nobody will read them then I will save my coworkers from having to listen to
boring stories about the origins of various bugs.

Bryan Cantrill wrote a tool in the latter part of his career at Joyent called
Statemap. I've written about statemaps in the past, specifically in 2019 when
we were looking at [PostgreSQL temp file IO](../2019/pg_temp_files.html),
and in 2020 when we used Statemaps to [debug pathological performance in MinIO](../2020/minio_tracing.html).
In fact, in the 2020 post about MinIO I alluded to writing about rust-statemap
and the manta-chum integration in the future.

To level-set again, we need to talk about how confusing the term 'statemap' is.
'Statemap' can refer to any of these three things:
1. An SVG file (a statemap rendering)
1. Lines of JSON data in specific formats (sometimes referred to as 'instrumentation output')
1. A program that turns JSON into an SVG (the statemap tool)

When we set out to build rust-statemap we wanted to do two things:
1. Define the statemap line protocol
1. Provide a library for Rust programs to use the statemap line protocol

The Joyent [statemap tool](https://github.com/joyent/statemap) includes
a private API and definitions of the statemap line protocol. When the statemap
tool was written I believe it was mostly intended for use with DTrace, though
the door was open for other forms of instrumentation. Since we were writing a
lot of Rust code (which is hard to instrument with DTrace, though I did create
[a PoC tool](https://github.com/KodyKantor/rdt) to add DTrace probes to
Rust programs), we needed a way to generate statemap instrumentation output
in a relatively idiomatic way.

Anyway, long story short I did a bunch of work to define a public statemap API
with Rust bindings to make it easy for Rust programmers to generate statemap
instrumentation output and called it [rust-statemap](https://github.com/KodyKantor/rust-statemap).

### manta-chum statemaps

The first time we used rust-statemap was for our 'minio statemap' tool, which I
already described in 2020. The second use was in
[manta-chum](https://github.com/joyent/manta-chum. manta-chum is a
load-generating tool the supports a bunch of common file protocols: s3, webdav,
and posix. It seeks to bridge the gap between micro-benchmarks like fio and
mega-benchmarks like COSbench.

manta-chum runs with a bunch of 'normal' synchronous threads (no green-threading
stuff, except where it couldn't be avoided), and we wanted to see what each
thread was doing during a benchmark. We wired up rust-statemap and gained a few
cool things:
- We knew the operations performed by each thread
- The duration of each operation is recorded
- We can consul, and slice/dice the above two data points at any time
	and in any way

Take this example run of manta-chum on my laptop, below. In this test we ran
manta-chum with its default settings: 1 thread, mix of read/write operations,
writing to the local filesystem with fsync enabled after each write. Feel free
to use the + and - magnifiers and click on the graph to interact with the
statemap. I suggest zooming in to discover what happens between the blue 'fsync'
operations. Each line on the y axis (there is only one) represents a thread,
and the x axis represents time beginning when the manta-chum process spawns and
ending when the manta-chum process finished writing the statemap data file.

The 'light' theme is best for viewing these statemaps.

<iframe src="assets/single-chum-thread.svg" title="single thread" style="border:none;" scrolling=no width=1024
	height=400></iframe>

If you zoom in and pan around you can see that this single thread is spending
all of its time issuing fsync syscalls, 24ms at a time. Then between fsync
operations the thread is issuing read, write, open, and mkdir calls, all of
which finish quickly.

This is interesting, but what happens if we have a bunch
of threads doing IO at the same time? Here's another statemap rendering from
manta-chum, this time with 10 threads running at the same time:

<iframe src="assets/ten-chum-threads.svg" title="ten threads" style="border:none;" scrolling=no width=1024
	height=400></iframe>

There is a lot more happening in that statemap. Notably, everything is much
slower. open, fsync, and mkdir calls can all take over 100ms each. The read
syscalls are still nearly instant.

Now, what happens if we remove that pesky fsync call? Let's run the same test,
but with fsync disabled.

<iframe src="assets/ten-chum-threads-no-fsync.svg" title="ten threads no fsync" style="border:none;" scrolling=no
	width=1024 height=400></iframe>

Bonkers! This time the ten threads wrote 5GB in 2 seconds, but the previous two
runs wrote ~200MB in 16 seconds. Of course the 5GB written during this test is
just buffered in memory. This statemap rendering shows a lot of interesting
things that raise a lot of questions:

- Why is there an occasional ~6ms pause across all chum threads?
- Why do read operations suddenly take much longer after 1.2 seconds of benchmark runtime?
- Approximately every second all of the mkdir and open calls take a long time across all threads. What gives?
	

### A new hope (for the future)

In the end we have been happy to have the rust-statemap library for learning
more about how our rust applications work. Along the way we found a bunch of
rough edges with the statemap protocol. Rather than repeating them all here,
check out the list of 'Statemap protocol moans and niggles' in the rust-statemap
repository's [README](https://github.com/KodyKantor/rust-statemap/blob/master/README.md#statemap-protocol-moans-and-niggles).

Unfortunately manta-chum helped us to realize that we couldn't bring our storage
product up to a level of performance that would compete with newer storage
systems. manta-chum was the last piece of Rust code I anticipate working on for
quite a while. If I end up writing more rust in the future I would love to
address these statemap protocol issues so it's even easier to create statemaps.
