# Owen Hash Experiments

This is the code used for the experiments and hash-searching process as discussed in https://psychopath.io/post/2021_01_30_building_a_better_lk_hash

Although I have made some effort to clean up the code, it is nevertheless very much "research" code, and not really meant to be good quality or robust.

The code in the `burley-scrambling-suppl` subdirectory is a modified version of the supplementary code from the paper [Practical Hash-based Owen Scrambling](http://jcgt.org/published/0009/04/01/).  The modifications are mostly just me adding the scramble approaches from the above-linked blog post.  But I did also update it to Python 3 to be able to use it on my system, since some of the necessary libraries are no longer available for Python 2 on Ubuntu Linux.

Other than the code from Burley's supplemental material and the direction number files in `direction_numbers`, all the code in this repo is dedicated to the public domain through [CC0](https://creativecommons.org/publicdomain/zero/1.0/).