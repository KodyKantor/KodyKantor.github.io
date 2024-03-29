# Dockerbox - Virtual Filesystem
#### 2017-01-17

One of my favorite recent projects is `dockerbox`. `dockerbox` was a
fun little idea I had after having messed around with Docker for the last couple
years. [Docker](https://www.docker.com/) is a few different things -
a REST API wrapper around container primitives in the kernel, a packaging
mechanism for binaries and their dependencies, a process manager, and a bunch
of other stuff.

I thoroughly enjoy spending most of my day in the Linux CLI. Something that
bothers me about Docker is that the CLI is sort-of like the Linux CLI.
Specifically, there's a `docker ps` command to list processes (containers), and
a `docker exec` command to spawn a process inside of a running container.
`docker exec` is a catch-all command, so you can run whatever CLI program you
want to inside of the spawned container. It's really not hard to use the Docker
CLI. It has a very slight learning curve. Like most CLI programs, there's a near
infinite number of commands and flags to control the program.

`dockerbox` aims to provide the familiar Linux CLI to managing containers. If
I'm on Linux (or Unix) and I want to see my running processes, I can do
something like `ls /proc`. That will list out the process IDs for running
processes, and I can drill down from there. Here's something that you might see
on a Linux host:


```
$ ls /proc
1/  10/ 13/ 25/ 138/  139/  1130/
kmsg  interrupts  meminfo
...
```


If I wanted to, I could run `ls /proc/1/` to see information about process 1,
like open files, cgroup stuff, and more. `/proc` is a virtual filesystem with a
[cool history](https://blogs.oracle.com/eschrock/entry/the_power_of_proc).
`/proc` may be getting a
[rewrite](http://2016.texaslinuxfest.org/sites/default/files/slides/time-to-rethink-proc-160821145019.pdf)
in future versions of Linux to fix some of the performance issues that are
appearing as a result of new tools like [CRIU](https://criu.org/Main_Page").
CRIU in particular does a ton of open-read-close calls on the files in `/proc`,
which are expensive operations. After taking into consideration that CRIU freezes
applications while it is running means that speed of collecting process
information is crucial to minimizing application downtime. With the changes
being suggested, `/proc` may be going back to resemble its [original form](http://lucasvr.gobolinux.org/etc/Killian84-Procfs-USENIX.pdf)!

`dockerbox` aims to provide similar functionality for containers. So like `/proc`
but for container filesystems. If I were running `dockerbox` on a machine, I
could do something like this:

```
$ ls /containers
happy_hopper/ pensive_curran/ amazing_goldwasser/ admiring_kowalevski/
```

Each of the 'directories' listed is a container. Running containers are colored
green, and stopped/exited containers are colored red. Since those are
directories, I can do something like this:

```
$ ls /containers/happy_hopper/
bin/   dev/   etc/   home/  proc/  root/  sys/   tmp/   usr/   var/
```

The equivalent statement in Docker lingo is `docker exec -it happy_hopper ls /`
which doesn't exactly roll off the tongue, but is simple once you get over the
slight learning curve.

A really annoying part of using containers for the test/dev cycle is that
containers... contain! There isn't a straightforward way to do things like copy
files from one container to another. With `dockerbox`, it's trivial!

```
$ ls /containers
happy_hopper/ pensive_curran/ amazing_goldwasser/ admiring_kowalevski/
$ cp /containers/happy_hopper/etc/fstab /containers/pensive_curran/tmp/other_fstab
```

Ok, so that example isn't implemented, but it's possible! The best thing about
the CLI is the possibility for an [orgy of
one-liners](https://www.princeton.edu/~hos/frs122/precis/mcilroy.htm)! With
`dockerbox`, that can be a real possibility.

Another thing that I wanted to implement was making this cluster-aware. Nobody
really talks about making something work manually on a single machine. There is
already some primitive multi-host usage in `dockerbox`, but nothing great. It
would be cool if we could see something like:

```
$ ls /containers
host1/  host2/  host3/
$ ls /containers/host1/
happy_hopper/ pensive_curran/ amazing_goldwasser/ admiring_kowalevski/
```

or better yet:

```
$ ls /containers
app1/ app2/ app3/
```

but I don't think application-blueprint-aware scheduling exists and would take
a long time to write.

With `dockerbox` in its current state, you can `cat` files, `cd` (but I had to
overwrite the `cc` command because `cd` is built into `bash`), `ls`, and some
other things. There's a fallthrough mechanism built in, so some commands will
work out of the box (pun intended) without any code needing to be written to
support it. I overwrote `ls` to do things like colorize running/stopped
containers. Generally anything dealing with the topmost layer of the virtual
filesystem (the `/containers` namespace) would have to be overwritten because
`/containers` doesn't actually exist unless this would be written in the kernel.


All of this is accomplished using the [Docker Remote
API](https://docs.docker.com/engine/reference/api/docker_remote_api/). The code
is on [GitHub](https://github.com/kodykantor/dockerbox). The bottom of the
README has a few other pie in the sky things as well. If it sounds interesting,
definitely check it out! It is definitely not good code, since it was hacked
together for a hackathon.