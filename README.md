# rust-voltorb-flip-solver
An efficient tool for solving Voltorb Flip puzzles.

No download link yet, features so far:  
<ul>
<li>Uses all cores</li>
<li>kinda performant</li>
<li>nice colorful graphic user interface, babyproof, worst you can do is restart the calculation</li>
<li>provably optimal (someone provided me with Nintendo's level generation algorithm)</li>
<li>displays the actualy win chance and, if you're curious, the changes of being a bomb/1/2/3 for each square</li>
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
<li>choose the yellow squares (=free coins), if there are none wait for the dark blue ones (=highest chance to win the ENTIRE puzzle)</li>
</ul> 

Note:
<ul>
<li>Maximizes the chance of winning the ENTIRE puzzle, doesn't care whether it dies in one or three moves</li>
<li>I'm still working on it, those problems will be solved, there will be suitable modes</li>
</ul>
