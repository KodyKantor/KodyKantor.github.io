+++
date = "2017-01-17T10:29:40-06:00"
title = "android cr"
description = "Thoughts about the viability of checkpoint-restore in Android"
categories = ["software", "android"]

+++

I got my first computer in 2004 for Christmas. It was some kind of Dell tower
with an awesome flat screen LCD display! It was given to me in the evening, but
for some reason I couldn't set it up immediately and I had to wait until the
morning. When the morning came I got up, and in a flurry of excitement pulled
all of the parts out of the box and put it all together. It had a single core
Pentium IV processor, and a whopping 128MB of RAM, integrated graphics, and a
killer DVD drive. When I had it all put together I clicked the power button and
heard the fans start up. The lights on the tower came on, but nothing appeared
on the screen. I double checked that the screen was turned on, and hard-reset
the tower. Same thing. I think I stared at the blank flat screen monitor for a
solid ten minutes before giving up.

I went to find my dad to help me figure out why the darn thing wasn't working.
It turns out that I had neglected to connect the monitor to the tower! A silly
mistake that only a fifth grader would make!

After a while my brother (who had an identical computer) decided to get a different
computer, so I took the RAM out of his and stuck it in mine. Math lovers will
note that I now had 256MB of RAM. I thought that was a lot back then! In 2011 I
bought parts to put together a computer, and bought 16GB of RAM for something
like 20 bucks. Now my phone has 2GB of RAM, and it seems like it's always starved
for memory.

Android, Windows Phone (if that's still a thing), and iOS are all used pretty
much the same way. You can run one application in the foreground at any given
time, and the rest of your applications idle in the background, or are killed if
the system is starved for RAM. For me, it seems like applications are constantly
being killed to provide RAM for the process in the foreground. This makes
multitasking really cumbersome and difficult. As a result of the apps being killed,
I have to wait while they reload when I open them, and all of my session state
is missing. Is there any way to solve this problem?

One of the things that I was involved in recently was working with some interns
on implementing live migration of bare-metal processes across machines. We used
a Linux tool called [CRIU](https://criu.org/Main_Page)(checkpoint/restore in userspace) which has the ability
to checkpoint and restore processes. Even though it's called '...in userspace,'
it has required a number of changes to the Linux kernel, like enabling the kernel
to track process memory changes, and a fork_with_pid() function that requests
a specific PID to be used for the new process. They're also considering changing
the way `/proc` is designed. `/proc` is used very heavily during the C/R
process, so the overhead of opening/closing all of those files is quite large.
In the case of live migration, any slowdown in CRIU represents an increase in
the application downtime, which is not good.

Most people think that C/R is only useful for live migration. That's definitely
not the case! When I was thinking about my phone's RAM problem, I
thought that it could be an interesting use case for C/R! Android runs on a
modified Linux kernel, so it may have the ability to run CRIU assuming the Android
maintainers are merging from upstream (which I believe they are).

The question is, what can we do with C/R in Android? Let's say that I just woke
up. I like to read the news, check my email, check my calendar, and then
maybe play a game (nothing like being lazy to start the day!). Maybe I
get a Snapchat notification while I'm playing this game. I switch to the Snapchat
application, reply to the snap, and then switch back to my game. When I switch back
to my game I have to completely restart because it was killed! Snapchat uses most
of the RAM on my device, and so does the game! I'm
frustrated now, so I switch back to Snapchat. It turns out that Snapchat was
killed while my game was starting! AHHHH! They can't both be in memory, so now
I have this vicious cycle of frustration.

C/R to the rescue? What if we could checkpoint an application (that we know uses
a lot of RAM) when it is taken out of the foreground? Then we don't need to worry if it
gets killed. All of the state (including registers, TCP connections, memory, etc.)
is on the internal storage. We can use Snapchat all that we want. When we need to
go back to our game, the system issues a restore from storage. All of the pages
are mapped back into memory, and we're off to the races! The game is EXACTLY where
we left it. While we're playing the game, Snapchat is checkpointed so we can return
to that without having to wait through the agonizing startup routines again.

Now let's say that I was in the middle of beating a boss on my video game when
I get a phone call. The phone call forces my game to be checkpointed and killed.
Maybe the phone call was a nasty hacker and they used some weird key tone that
causes my phone to reboot. Damn it! I was JUST BEATING A BOSS. My phone reboots,
memory is wiped, and I frantically open the game. Luckily, the game was just
checkpointed, so it was restored to the point before I got the malicious phone call!
When I open up the game, I'm right back to where I was: in the middle of an
epic boss fight.

There are a ton of problems with thinking this will work. TCP connections (if
any - I'm not sure that mobile apps maintain long TCP connections) will probably
be dropped by the server in the time between checkpoint and restore. Android is
hella fragmented, and that fragmentation goes all the way down to the kernel.
I'm pretty sure manufacturers NEVER update the kernel on their devices. So any
changes to fix or enhance the kernel features of CRIU would probably never be
sent out to phones.

Luckily mobile applications don't use much memory, so we shouldn't have to worry
about copying many GBs of memory to storage when a checkpoint occurs. However,
phones also don't usually have much extra storage. Another problem would be the
power usage. I'm guessing dumping memory isn't the most efficient of tasks.

There are some cool things about this. Applications could use more memory without
impacting other applications. Applications persist state across reboots. This
approach is pretty much only viable as a thought exercise because mobile users
only interact with one application at a time.

Even though it isn't more than a thought, it's still fun to think about the use
cases of C/R on Android.
