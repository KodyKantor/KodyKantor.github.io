# Hardware RAID is Lying to You
#### 2019-03-08

We were migrating a large ZFS filesystem (11.4T logical) from one machine to
another. I was warned ahead of time that the old system was _very_ old:

- It was running the `20141030T081701Z` platform image.
- uptime(1) states: `up 1521 day(s)` (wow!).
- ZFS is sitting on top of hardware RAID.

We haven't used this configuration in years:
```
NAME STATE READ WRITE CKSUM
zones ONLINE 0 0 0
c0t0d0 ONLINE 0 0 0
```

The machine is running a version of ZFS so old that the spa doesn't have
'ashift' members, meaning it is difficult to accurately guess what the sector
size of the drives are assumed to be. Since this is so old we can probably
safely assume they are 512b sectors behind the RAID controller.

Anyway, the problem was that we had numerous 'zfs send | zfs receive' failures.
The reason for this was that the received dataset was going well over the
storage quota assigned to the dataset. The dataset was assigned a 15T quota and
has an 8K record size.

At first it seems strange that this would happen. Why would an 11.4T dataset hit
a 15T quota? Well, the folks running the dataset migration decided to turn off
the quota, thinking that it can't be _much_ more than 15T. Maybe some extra data
is used when large datasets are transferred or something.

So a couple days later we realize that the migration is _still_ running and the
dataset on the target machine is now 21T! WOW! That's almost double the size of
the original dataset!

What's all that extra data, where is it coming from, and what the hell
happened?!

What if I told you that this is a feature, and not a bug? Let's take a look at
the target system to figure out why.

- Running the 20181206T011455Z platform image.
- uptime(1) states: up 53 day(s)
- 11-wide RAIDz2 with a SLOG device and one spare:

```
NAME STATE READ WRITE CKSUM
zones ONLINE 0 0 0
raidz2-0 ONLINE 0 0 0
c3t5000CCA25329A52Dd0 ONLINE 0 0 0
c3t5000CCA25330316Dd0 ONLINE 0 0 0
c3t5000CCA253306FEDd0 ONLINE 0 0 0
c3t5000CCA253361779d0 ONLINE 0 0 0
c3t5000CCA253380975d0 ONLINE 0 0 0
c3t5000CCA25346BA21d0 ONLINE 0 0 0
c3t5000CCA2534BC35Dd0 ONLINE 0 0 0
c3t5000CCA25353DC69d0 ONLINE 0 0 0
c3t5000CCA253543865d0 ONLINE 0 0 0
c3t5000CCA2535471B5d0 ONLINE 0 0 0
c3t5000CCA253556625d0 ONLINE 0 0 0
logs
c1t4d0 ONLINE 0 0 0
spares
c3t5000CCA253CBD0A5d0 AVAIL
```

This is a very new box. As far as I know we haven't deployed anything to it
previously. Its disks have 4K sectors.

Let's think about what happens when we write a block to ZFS on the old machine.

- The application writes an 8K block.
- ZFS (in RAID0 mode) writes an 8K block.
- Hardware RAID does magic junk that us mere mortals are not privy to.

These magic things at least include writing parity blocks for
however the parity is configured. It's hard to tell what is
happening or how this is configured because it's all supposed to
be magical happiness and good times. We may even have to enter
into the BIOS to see what the RAID settings are (which would
mean ending the 4+ year uptime track record!).

- ZFS believes it wrote 8K of data both logically and physically.

Now let's think about what happens when we write a block to ZFS on the new
machine.

- The application writes an 8K block.
- ZFS (in RAIDz2 mode) writes an 8K block and two parity sectors (4K).
- ZFS believes it wrote 8K of logical data and 16K of physical data.

Do you see where the extra data is coming from now? It's the parity!

After we discovered this we let the transfer finish, and now the _logical_
dataset sizes match while the _physical_ dataset size on the target machine is
double that of the source machine:

Source:
```
zones/b7c55652-e7c8-46b0-9f57-fae90314caf5 used 11.7T
zones/b7c55652-e7c8-46b0-9f57-fae90314caf5 logicalused 11.6T
```

Target:
```
zones/b7c55652-e7c8-46b0-9f57-fae90314caf5 used 26.6T
zones/b7c55652-e7c8-46b0-9f57-fae90314caf5 logicalused 11.6T
```


As I was looking into this I also noticed that we have lz4 compression turned
on, but it isn't doing anything other than increasing latency and burning CPU
time:

```
zones/b7c55652-e7c8-46b0-9f57-fae90314caf5 compressratio 1.00x
zones/b7c55652-e7c8-46b0-9f57-fae90314caf5 compression lz4
```

I didn't quantify time wasted trying to compress data. It could be that the
application is doing compression before hitting ZFS, or the data being written
in incompressible.

We learned a few things from this:

- Hardware RAID hides information from advanced filesystems like ZFS.
- Hardware RAID is difficult to debug and gather information about.
- Hardware RAID makes it seem like you're using less storage capacity than you
truly are.
- Using an 8K record size on RAIDz2 gives you 50% disk efficiency (although
you are given net more storage than disk mirroring due to decreased
durability).
- Make sure ZFS compression is doing _something_ if it's enabled.