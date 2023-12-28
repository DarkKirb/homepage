---
title: Sims 2 Scripted Events
tags:
changed: 2023-09-03 09:24:33
---

The Neighborhoods that have shipped with The Sims 2 Basegame include mini-tutorials in the form of scripted events for the premade playable sims. These scenes are meant to teach the players various game mechanics from marriage, over cheating, to alien abductions. These events were not present in more recent neighborhoods.

This article will describe what these scripted events are supposed to do, what happens in-game, an unused event, how scripted events are programmed in the game, and why this might have made it harder to add scripted events to new hoods.

<!--more-->
<!-- toc -->

I’ll be looking at the object “Controller - Game Scripting” (GUID `0x8D0C081F`), from the base game of The Sims 2: Ultimate Collection. This is also going to apply as is to the macOS and disc versions of the game. Newer expansions introduce additional scripts for the demos that were showcased at E3 or on the Home Shopping Network.

These are in order of the script instance IDs, potentially the order these scripts were added into the game.

## How the scripts work

The Game Scripting controller is a global object, meaning that it will spawn automatically when you enter a lot. It is invisible, and can’t be directly interacted with, although interactions are defined, and used by the script for the Baxter family. The first thing it does is wait until the move-in cinematic completes. Before any script is run, the controller will check if it has run before for this household, and if it hasn’t it mark the household as “script completed”.[^1] If it has previously run it will automatically delete itself.

[^1]: It adds the token “Controller - Game Scripting” to the household’s inventory.

It figures out what script to run by checking the current neighborhood ID and lot ID. For example, the Pleasant Family live in 215 Sim Lane in Pleasantview. Pleasantview is neighborhood ID 1 on a fresh save directory, and 215 Sim Lane has the lot ID 68. Therefore the script checks that the current neighborhood has ID 1, and the current house has ID 68, before it will execute the Pleasant’s scripted events. Right before it executes the scripted event, it will disable the welcome wagon that might arrive otherwise.

Some effects of these scripts are implemented in other objects. Jobs and School let you force bringing home a guest. Telescopes of either size let you force an alien abduction. The chance card in the Pleasant scenario, and the promotion in the Beaker scenario are special, because in those cases the game will check for this controller’s existence, and check if it wants to force a chance card or a job promotion. It even lets you force an always successful chance card, which is seemingly unused.

## Pleasant Family (Pleasantview)

- Wait for 40 sim-seconds
- Makes it so that when Mary-Sue is at work, a chance card appears, which always fails. This is done by having the chance card code check for the scripting controller. The output will always be predetermined. In this case the chance card will fail.
- Find Mary-Sue Pleasant, store reference to her
- Find and break the cheap TV
- Finds Angela Pleasant, makes sure she is a teen, and resets her school grade to B.
- Finds Lillith Pleasant, makes sure she is a teen, and resets her school grade to D-.
- Finds Dustin Broke, and makes sure he is a teen.
- Finds Daniel Pleasant, stores reference to him.
- Shows the Social Need desperation bubble above Daniel’s head, and then the message: “Who says good help is hard to find? Daniel has his eye on the maid and wants a little romance, but will his decision threaten family unity?”
- The game then waits until Mary-Sue leaves the lot, checking every two Sim-minutes
- The script forces Angela Pleasant to bring Dustin Broke home.
- It will now wait up to 16 hours for Mary-Sue to catch Daniel cheating, and if she does the game will forces Cassandra Goth to appear 8 minutes later.

Trivia: The message doesn’t use Daniel’s name directly, so if you change it it will appear differently here, unlike with other scripted events.

## Goth Family (Pleasantview)

- Don Lothario is already on lot, before the script starts
- The game resets the needs of all of the sims on the lot.
- It will select Cassandra Goth and show the message: “Cassandra is smitten with Don and wants to get hitched. Perhaps an affectionate gesture will put the relationship on strong footing.”
- The script then waits until Cassandra kisses Don in any way. If she kisses Don while they are still engaged, a popup will appear that reads “Ooh! Fireworks! That was quite a kiss! Cassandra better strike while the iron is hot and get married. Click on the wedding arch to get the ceremony started.”. This does not happen if they are no longer engaged (married or broken up)
- After 19:00, the game will force every single grave to spawn a ghost. This happens even if you haven’t kissed Don.

## Lothario Family (Pleasantview)

- Checks if Dina Caliente [!] is dead. the script will abort if she is dead.
- The game will select Don Lothario
- It will then find the telescope on the roof, to make it so that if he looks through the telescope at day, he will have Dina Caliente come over angrily.
- Displays the message “Don’s romantic life is really humming along! Maybe now’s the time to put the moves on Nina Caliente. Pick up the phone to invite her over.”
- It will check if there is a Maid on the lot, or if a Maid is scheduled to come. If neither are, and Don has a trash bag in his hand the game will tell you: “This place is a mess! Use the phone to hire a maid.”
- If the clock hits 22:00, the scripted event will cancel
- It will then wait for Nina to arrive, and when she does, the message “Don wants to win Nina over, and it looks like she’s slowly succumbing to his wily charms. Maybe now’s the time to suggest a romantic soak in the hot tub!”
- Afterwards, Nina’s needs will be reset regularly

## Broke “Newbie” Family (Pleasantview)

- Looks for Dustin, stores it, and checks if he’s a teen
- Switches to Brandi
- Looks for Beau, stores it
- Shows the message “Beau Broke is growing fast, and Brandi wants to teach him all the skills he needs to stay out of trouble. Click on Beau to teach him to Walk,and fulfill wants for both Sims.”
- Forces Dustin Broke to bring home Angela Pleasant
- Waits until Dustin Broke has returned home from school
- Makes Brandi Broke lecture Dustin Broke about either Reading, Arithmetic, Geography, or Science (it’s random!)
- At 18:00, it will spawn in Angela, and start a phone call about her wanting to sneak out. It will set the visitor type to 0x16, and misc flags to 0x13 (off-world loiterer currently on a phone call?)

## Curious Family (Strangetown)

- Looks for Vidcund Curious, selects them
- Shows “Vidcund wants to explore the heavens, but his new telescope might bring him a little too close for comfort!”
- Between 5:00 and 22:00 it counts the number of babies in the current family. If there was at least 1 baby, show “Phew! Interstellar spawn sure are tiring - maybe your Sims could use a little help. Pick up the phone and call a Nanny!”. Then it spawns Nervous Subject. It then pushes 0xED2A5AA2 (presumably meant to be Pascal Curious) to greet him.
- Between 18:00 and 5:00 if one of the two expensive telescopes is used, it will force the abduction to happen. It doesn’t matter who uses them, however it will only happen once.

## Smith Family (Strangetown)

- Looks for 0x6D469423 (Johnny Smith), verifies that he is a teen, focuses on him, and shows the message: “Johnny’s ready to become a man! He wants a birthday party to mark the occasion, so pick up the phone to invite his friends. Blow out the candles when everybody’s ready!”
- It then waits for a party to start, and once it does it will check if 0x0036 is alive, and if he is spawns 0x6D2FC195. If he leaves the lot in any way, the game will respawn him.
- Once Johnny ages up to an adult the game will show: “Johnny is ready for a life of his own, getting a job and moving out will start him on his adult journey.”

## Specter Family (Strangetown)

- Looks for 0x4D2E591C (Ophelia Nigmos) and verifies that she is a teen
- It then sets the motifs of 0x6D2E5338 (Olive Specter) and Ophelia.
- Then it shows the message “Olive Specter wants to retire! Just pick up the phone and call work to start collecting that pension.” and focuses on her
- Sets the “Last Hour Processed” for the aging controller to 0xFFFF
- It then locates a Teddy Bear and Xylophone and saves it for later
- At 20:00, for each “UrnStone - Moderate” on the lot, it will spawn the ghost, wait until it does something, and make it haunt the xylophone(?)

## Caliente Family (Pleasantview)

- Looks for NID 1 (Mortimer Goth), checks if he’s still alive
- Sets the Motives for 0xCDAE8902 (Dina Caliente), and 0x2DAE885B (Nina Caliente)
- Selects Dina Caliente
- Makes a Burgular spawn at night
- Shows message “Dina Caliente’s digging for gold, and she wants to lay a claim on Mortimer! Don’t wait for him to make the big move! Be aggressive!“
- It then spawns 0x4DAE6810 (Mortimer Goth)
- Once it detects that Mortimer Goth has joined the family (in any way), it will check if 0x000E (Nina Caliente) is alive, and show the message: “Two’s company but three’s a crowd so it’s time for Nina to get moving. Use the newspaper to move a Sim out.”

## Dreamer Family (Pleasantview)

This script is quite simple. When you load the lot, there are already 3 sets of white bills lying around the lot (an amount that is not possible during normal gameplay). Once you unpause, the game will select Darren Dreamer, and show the message “Darren Dreamer wants to paint for a living, but does he have the skills to pay the bills? Those past-due statements are starting to pile up…”.

The script now waits for a painting to be completed on the easel upstairs. It doesn’t check that Darren does it, so in principle Dirk could do it instead. Once that is done, it shows the message “Still haven’t found your artistic voice? Try painting a still life or a portrait of a family member to add a personal touch to your artwork.”

## Grunt Family (Strangetown)

- Finds the Job Finder Controller
- Sets the attribute 0x0000 to the current day of month
- Finds and deletes the newspaper
- Looks through all of the teen/elder jobs and only marks the millitary career as available
- Marks the millitary career as available, and all
- Finds 0x6D2FC195 (Tank Grunt?), Focuses on him and shows the message “Young Tank really wants his father’s approval. Will finding a good job and working hard finally win Tank his father’s respect? Use the computer to start the job search.”
- When tank joins a job, 0xED2FC2F1 will perform action 0x47 on Tank
- The scenario will end either when Tank joins the millitary career or becomes an adult.

## Beaker Family (Strangetown)

- It sets the “Force Job Promotion” flag. I do not know how it works exactly
- Searches and selects 0xAD2E610C (Circe Beaker?)
- Shows “Circe wants a big promotion! Get her to work in a good mood, and maybe her efforts will pay off!”
- The script will wait until Circe leaves the lot. The script is aborted if she dies.
- It then searches for the Fridge, and 0xCE06B376 (Nervous Subject), and makes Nervous Subject perform action 0x000B on the fridge. The interaction will keep getting pushed until it completes.
- Once the fridge is being used, the script will spawn cockroaches next to Nervous Subject.
- The script will continue running until 23:00, doing nothing.

## Baxter Family (Waterside)

This is the first hidden scripted event, used as a Demo at E3 2004

- Stores this script as LocalA
- Finds 0x4DA53229, stores it as Local0 (Don Baxter)
- Finds 0x0DA531DC, stores it as Local1 (Sarah Baxter)
- Finds 0x6DA17DAD, stores it as Local2 (Dina ???)
- Finds 0x8DA17B35, stores it as Local3 (Virginia ???)
- Finds 0xCDA53259, stores it as Local4 (Alex Baxter
- Sets LocalB to 0
- Waits until Sarah snuggles Alex
- Waits until Alex becomes a child
- Finds the Food Buffet table, stores it as Local5
- Creates a pathing destination, placed near the Food Buffet Table, stored as Local6
- Makes Dina and Don go to the Buffet table
- Sets the attribute 1 of Alex to 3
- Pushes Alex to go to the telescope
- Waits until Don and Dina are chatting
- Makes Dina Gussy up on the Loft Mirror
- Waits until don performs social interactions on a sofa
- Makes Sarah Baxter stand near the food table until Virginia is distressed
- Makes Virginia go to the Loft Mirror
- Waits until Virginia is Indoors
- Waits until Virginia is distressed
- Makes Sarah do Action 0xF0 on Virginia
- Wait until Sarah gossips
- If the state of person B in that interaction is 0xFFFF or not 9 or 0xC, go back two steps
- if the state of person B is 0x0009 or 0x000C, make Dina go to the food table.
- The game then waits for Don to apologize and the interaction has attribute 6 above the value of 4
- It will then try to make Dina perform a romantic kiss on Don, and when the state of person A in that interaction is above the value of 4, it continues
- It will then wait until Sarah’s Aspiration Score goes below 0, and then above it again.

{% youtube _u2zoTsaf-Q %}

The above scenario being shown by Tim LeTourneau during E3 2004

{% youtube RpItHVR9oa0 %}

The Baxter Family being shown by Bill Wright at E3 2004

{% youtube t1ylnWOgGB0 %}

A Fanmade recreation of Waterside and the Baxter Family, including part of the scenario


## Capp (Veronaville)

- Find Juliette Capp, verify that she is a teen
- Find 0x4D5C9759, set their motives
- Set Juliette’s motives (local0)
- Find 0x8D5DF0A5, set their motives
- Find 0xAD5CEF3A, set their motives
- Find 0x8D5DF0A5, check if they’re a teen (local1)
- Find 0xAD5CEF3A, check if they’re a teen (local2), selects them
- Show: “Juliette really wants to make things official with Romeo, but her grandfather might not approve. Can they pull off Going Steady without Consort noticing?”
- Makes local1 play noogie on local2
- Find 0x8D5C4AF, check if they are a teen (local3)
- Waits until local3 has been greeted and local0 went steady with local3
- Makes local3 kiss - peck local0, wait until it completes
- Find 0x4D5C9759 (local4), have him give local3 a strong lecture, wait until it completes
- Have local3 do interaction 0 on the Portal - Pedestrian
  
## Monty (Veronaville)

- Find 0x8D5CE4AF, checks if they are a teen (local1)m sets their motives
- Find 0x6D5CE683, sets their motives (local2)
- Find 0x2D5CAB86, sets their motives
- Find 0xAD5CADA0, sets their motives
- Selects local1
- Checks if local2 is a teen
- Local4: Portal - Pedestrian
- Show: “Romeo is dying to see his main squeeze. He wants to invite Juliette over for a make-out session. Pick up the phone and give her a call!”
- Wait until 0x0D5C9DAB is on the lot (local0)
- Check if they are still a teen
- Check if 0x8D5DF0A5 (local3) is on the lot, if not spawn him, and make sure he’s a teen
- Waits until local1 kisses local0 and the interaction is finished
- Makes local2 insult local0
- Once local2 is no longer insulting local0, local3 fights local2
- after that local1 will lecture local3
- Makes local3 perform action 0 on a flamingo
- makes local3 leave

## Summerdream (Veronaville)

- Finds 0x8D5E1391, check if they are a teen
- Finds 0xED82EE7F, sets their motives
- Finds 0x0D5E125F, sets their motives
- Finds 0x8D5E1391, sets their motives
- Finds 0xADB8FEF6, sets their motives
- Finds 0x8D5E1391, shows “The party is already underway, and Puck’s sweetheart Hermia is her! He wants that First Kiss, and now is the perfect time to make his move.”
- Sets the Temp0 of the party controller to 0xFC19 which appears to hide the text box.

## Curiosities/Oddities

There is actually a second related controller, named “Controller  - Motive/NPC Handler”[^2] in the same package. It spawns in the same way as the “Controller - Game Scripting” object. In the retail game it just deletes itself shortly after spawning, but looking at the code, it was intended to keep the motives of NPCs in check while scripted events take place.[^3]

[^2]: GUID `0xADA3DB7B`
[^3]: In particular, it sets the Bladder/Energy/Hygiene/Comfort/Hunger/Social motives to 45 if they drop below 40, and the Fun motive to 35 if it drops below 30.

The Scenario for the Broke family is internally called “Newbie”.

## Closing Remarks

So why were no scripted events included in later neighborhoods? I believe that additional scripted events were never planned, due the way the code is structured. As mentioned above, the code checks for the neighborhood ID, and every time a new neighborhood gets added, it receives a new ID. What ID it receives was predictable in the basegame, since the neighborhoods were copied into your Sims 2 folder before you could possibly add other neighborhoods. After that, however, you are free to add or delete neighborhoods as you wish, which affects which IDs additional neighborhoods get.

It is possible however to implement the script differently, by checking for specific Sims existing in the neighborhood. Each sim (like every other object) has a GUID, which should be unique between sims. An example of this can be seen with [Chris Hatch’s Custom Events mod](https://modthesims.info/showthread.php?p=5557184#post5557184).
