# Mini Mystery

A murder mystery game inspired by the likes of Mafia and Secret Hitler.

## Important Note

It turns out that it's hard to progam a whole game, complete a final project,
and do a week's worth of sociology homework in a single weekend. For this reason,
I haven't had time to tweak the villager counts or create villagers with
special abilities that interact well with each other and make for interesting
and _varied_ gameplay. Thankfully, this isn't a game design contest!

## Gameplay

Mini Mystery has the same setup as a traditional murder mystery game: there are
some number of villagers, some with special abilities, and some murderers.
Every day, you work to figure out who the murderers are. At night, each murder
kills a villager. But there's a twist! You never interact directly with the
village: instead, you send out robots called "minis" to do your bidding.
Minis are fully programmable, and every day, you will be prompted to provide
a file containing the code to run a mini and a starting location for the mini.
Your goal is to kill all murderers before all the villagers die.

## The Village

As of right now, the village contains 6 normal villagers, 2 strong villagers,
2 afraid villagers, and 2 murderers. Normal villagers have no special
abilities. Strong villagers will survive a single attack from a murderer.
Afraid villagers will kill your mini if it visits them (more on working
with minis soon). Finally, murderers will kill and clear the logs of any minis
that visit them and kill one villager a night.

Villagers are numbered 1 to 12. Every time a murderer goes to kill a villager,
it will randomly pick to either attack a villager with a number above or below
its own. It will attack the villager closest to itself in the randomly chosen
direction. If no such villager exists (the murderer is 12 and it chooses up
or the murder is 3 and villagers 1 and 2 are dead, for example), the murderer
will not attack any villager.

## Minis

Minis run on a small set of instructions. As minis run, "events" can be added
to their event log. A list of all posted events will be listed once the mini
has stopped running (provided the murder did not clear the mini's event log).
In addition to the two programmable events (post register and post flare),
a "finished" event will be added to the end of a mini's event log if it finishes
cleanly (i.e. its program terminates, it is not destroyed or lost). The only
way for a mini to store information and provide arguments to its instructions
is through its single register, a u8 initialized to 0.

A list of all possible instructions can be seen below, followed by a few examples
of full scripts. In mini programs, all whitespace is ignored. Because the user does
not provide their own names, it's even possible to remove spaces between instructions
(I wouldn't reccomend it though).

### Actions

There are four basic instructions a mini can perform:
- Post register ("post register"): posts the current value of the register to the event log.
- Post flare ("post flare"): posts a PostFlare event to the event log.
- Detonate ("detonate"): instantly kills both the villager at the location in the register and the mini.
  This is how you go about killing murderers.
- Visit ("visit"): visits the villager at the number in the register. A mini will begin its life by visiting
  its starting location (provided by the user at runtime after presenting the mini's instructions).

### Operations

A mini can perform three operations to its register:
- Increment ("incr"): adds one to the register. In case of overflow (recall the register is a u8),
  the mini is destroyed.
- Decrement ("decr"): removes one from the register. In case of underflow (recall the register is a u8),
  the mini is destroyed.
- Set value ("set `u8`"): sets the value of the register.

### Condition

Minis can be programmed with conditionals. The basic syntax is
```
if `condition` { `instructions` }
```

- Villager is alive ("alive"): runs the instructions only if the villager the mini is currently located at is alive.
- Villager is dead ("dead"): runs the instructions only if the villager the mini is currently located at is dead.
- Register equals ("eq `u8`"): runs the instructions only if the register equals the given value.

### Repeat

Minis can be programmed with loops. Use "break" to break out of loops (caling break when not in
a loop will end the execution of the program). Repeat has built-in infinite loop protection;
a repeat will automatically terminate after so many cycles. The basic syntax is
```
repeat { `instructions` }
```

### Example Programs

The program will prompt you each day to provide a file containing mini code. My headcannon is that the
file exension for these programs is .mm (the initials of the game), but that's also the file extension
for objective-c++. You can also just use .txt if you'd like.

Because I haven't had time to get into the game design of Mini Mystery (see the note way above),
the following is a super super super common pattern for starting games which can instantly reveal if there is
a murderer in the first few villagers (we break early since, if we just keep going, we'll likely run into
a murderer, have our logs cleared, and gain no information).

```
set 1
visit
repeat {
    post register
    if eq 4 { post flare break }
    incr
    visit
}
```

This example is very similar, but is something you may see a few turns into a game.
```
set 6
visit
repeat {
    post register
    incr
    if dead { detonate }
    visit
}
```