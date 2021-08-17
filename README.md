# rust-voltorb-flip-solver

<p float="left">
  <img src="https://user-images.githubusercontent.com/63099057/129618132-7642ce71-a68b-41cb-852b-8ee9f10f6f06.png" width="400" />
  <img src="https://user-images.githubusercontent.com/63099057/129618351-7c640987-ff53-4360-9d2d-0f2dd872e077.png" width="400" /> 
</p>

An efficient tool for solving Voltorb Flip puzzles (https://bulbapedia.bulbagarden.net/wiki/Voltorb_Flip).

Features so far:  
<ul>
  <li>First correct Voltorb Flip solver (Popular ones didn't even ask for the level :D). </li>
  <li>Uses all cores</li>
  <li>Very performant (given the difficulty of the task)</li>
  <li>Nice colorful graphic user interface, missclick-proof, worst you can do is restart the calculation</li>
  <li>Optimal and correct, assuming perfect randomness (someone provided me with Nintendo's level generation algorithm)</li>
  <li>Displays the actual win chance and, if you're curious, the chances of being a bomb/1/2/3 for each square when hovering over it</li>
</ul> 

System requirements:  
<ul>
  <li>Tested on > 200.000 puzzles sampled from the actual game</li>
  <li>Needed less than 30 seconds and 3 GB RAM for the tuffest puzzle (i7-8750H: 6 cores, hyperthreading)</li>
  <li>Usually solves puzzles within a few seconds</li>
</ul> 

Control:  
<ul>
  <li>Click on buttons to advance their number by 1</li>
  <li>Pressing a number while doing so advances to that number straight away</li>
  <li>Program starts calculating automatically</li>
  <li>Choose the yellow squares (=free coins)</li>
  <li>If there are none wait for the dark blue ones (=highest chance to win, but possibly a bomb)</li>
  <li>Grey means "useless" e.g. can't be a 2 or 3</li>
  <li>The windows title tells you about ongoing calculations, whether the constraints are consistent and so on</li>
</ul> 

Note:
<ul>
  <li>Starts out with puzzle 1 of examples.txt (extremely easy puzzle)</li>
  <li>Entering the level is just as crucial as the other constraints!</li>
  <li>Maximizes the chance of winning the ENTIRE puzzle, doesn't care whether it dies in one or three moves (yet)</li>
  <li>I'm still working on it, those problems will be solved</li>
</ul>

Future Goals:
<ul>
  <li>More speed through smarter algorithm and better implementation</li>
  <li>Modes like
    <ul>
      <li>"Maximize the chance not to drop in level"</li>
      <li>"Win with 8" to also uncover 8 cards which the jump to the secret level 8 requires</li>
      <li>"Maximize chance to get at least x coins", if you just need a few more coins for your favorite prize"</li>
    </ul>
  <li> Preferring the safer square when multiple ones have the same win chance</li>
  <li>Uploading videos about the algorithm on my YouTube channel "TriceraTobs":<br>https://www.youtube.com/channel/UCuFm7Z4abH4q_El93bdpDQg</li>
</ul>

Download: https://github.com/Tobs40/rust-voltorb-flip-solver/releases
