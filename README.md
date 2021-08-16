# rust-voltorb-flip-solver
An efficient tool for solving Voltorb Flip puzzles (https://bulbapedia.bulbagarden.net/wiki/Voltorb_Flip).

No download link yet, features so far:  
<ul>
  <li>uses all cores</li>
  <li>very performant (given the difficulty of the task)</li>
  <li>nice colorful graphic user interface, babyproof, worst you can do is restart the calculation</li>
  <li>optimal and correct, assuming perfect randomness (someone provided me with Nintendo's level generation algorithm)</li>
  <li>displays the actualy win chance and, if you're curious, the chances of being a bomb/1/2/3 for each square</li>
</ul> 

System requirements:  
<ul>
  <li>needs 30 seconds for the hardest puzzles on my laptop's i7-8750H (6 cores, hyperthreading) </li>
  <li>less than 3 GB RAM even for the hardest puzzles</li>
  <li>usually needs way less RAM and solves puzzles within a second or less </li>
</ul> 

Control:  
<ul>
  <li>click on buttons to advance their number</li>
  <li>pressing a number while doing so advances to that number straight away</li>
  <li>program starts calculating automatically</li>
  <li>choose the yellow squares (=free coins)</li>
  <li>if there are none wait for the dark blue ones (=highest chance to win, but possibly a bomb)</li>
  <li>grey means "useless" e.g. can't be a 2 or 3</li>
</ul> 

Note:
<ul>
  <li>Starts out with puzzle 1 of examples.txt (extremely easy puzzle)</li>
  <li>Entering the level is just as crucial as the other constraints!</li>
  <li>Maximizes the chance of winning the ENTIRE puzzle, doesn't care whether it dies in one or three moves</li>
  <li>I'm still working on it, those problems will be solved, there will be suitable modes</li>
</ul>
