# rust-voltorb-flip-solver

<p float="left">
  <img src="https://user-images.githubusercontent.com/63099057/131417534-4c1874e8-1e33-4cc5-8f55-cf16fde952b7.png" width="800" /> 
  <img src="https://user-images.githubusercontent.com/63099057/131417642-df0c98d0-a996-40a9-bd6a-4d711db63263.png" width="800" /> 
</p>

Tool for solving Voltorb Flip puzzles (https://bulbapedia.bulbagarden.net/wiki/Voltorb_Flip).

Features so far:  
<ul>
  <li>First correct Voltorb Flip solver (Popular ones didn't even ask for the level :D). </li>
  <li>Many different modes so you can choose your favorite strategy!
  <li>Can use any number of processor cores</li>
  <li>Nice colorful graphic user interface, missclick-proof, worst you can do is restart the calculation</li>
  <li>Recommends the best move(s), assuming perfect randomness (someone provided me with Nintendo's level generation algorithm)</li>
  <li>Displays the actual chance of succeeding so you can get mad about how unfairly hard this game is</li>
  <li>Also displays the chances of being a bomb/1/2/3 for each square when hovering over it</li>
</ul> 

System requirements:  
<ul>
  <li>Tested on 209.885 puzzles sampled from the actual game</li>
  <li>Needed 65 seconds and 5 GB RAM for the tuffest puzzle in the most demanding 'WinEight' mode (i7-8750H: 6 cores, hyperthreading)</li>
  <li>That's extremely(!) rare though, usually solves puzzles within a few seconds and much less RAM</li>
  <li>You can always switch to another mode if it's taking too long
  <li>Even your oldest PC can handle the 'SurviveNextMove' mode</li>
</ul>

Control:  
<ul>
  <li>Click on buttons to advance their number by 1</li>
  <li>Holding down a key while doing so advances to that number straight away</li>
  <li>Program starts calculating automatically</li>
  <li>Choose the yellow squares (=not a bomb, could be a 2 or 3 --> free coins)</li>
  <li>If there are none wait for the dark blue ones (=highest chance to win, but possibly a bomb)</li>
  <li>Grey means "useless" e.g. can't be a 2 or 3</li>
  <li>Note that for some modes yellow/gray squares don't make sense and thus aren't displayed
  <li>The windows title tells you about ongoing calculations, whether the constraints are consistent and so on</li>
</ul> 

Note:
<ul>
  <li>Starts out with puzzle 1 of examples.txt (extremely easy puzzle)</li>
  <li>Entering the level is just as crucial as the other constraints!</li>
  <li>If you don't know what you're doing I'd recommend the 'SurviveNextMove' mode, it's the default for a reason</li>
  <li>Modes are described in the GUI of the program, if you understand the Voltorb Flip game mechanics you'll understand why each mode exists</li>
  <li>Don't be scared to switch between modes anytime, the results of other modes are cached, the performance penalty is very small
</ul>

Future Goals:
<ul>
  <li>Uploading videos about the algorithm on my YouTube channel "TriceraTobs":<br>https://www.youtube.com/channel/UCuFm7Z4abH4q_El93bdpDQg</li>
</ul>

<br>
<br>

Download: Wait a few minutes
