**Your computer can double up as a cosmic ray detector. Yes, really!**

[Cosmic rays](https://en.wikipedia.org/wiki/Cosmic_ray) hit your computer all the time. If they hit the RAM, this can [sometimes cause disturbances](https://en.wikipedia.org/wiki/Soft_error#Cosmic_rays_creating_energetic_neutrons_and_protons), like flipping a random bit in memory.
To use your computer as a cosmic ray detector, simply run this program!  
The detection works by allocating a vector of zeroed bytes and then checking regularly to see if they are all still zero. Ta-da!  

 * Do not run this on a computer with [ECC memory](https://en.wikipedia.org/wiki/ECC_memory), as that will prevent the data corruption we are trying to detect!
 * The chance of detection increases with the physical size of your DRAM modules and the percentage of them you allocate to this program.
 * Beware of operating systems being clever, and e.g. compressing unused memory pages or swapping them to disk. A vector of nothing but zeros that hasn't been used in a while is an excellent target for this. This will shrink your detector!
 * Expect detections to be *very* rare.

It may also not work on DDR5 memory modules and later as those contain onboard ECC.

**Special thanks to**
 * /u/csdt0 and /u/HeroicKatora on reddit for ideas about how to improve the correctness of the program and avoid the pitfalls of virtual memory.

<br>

### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>