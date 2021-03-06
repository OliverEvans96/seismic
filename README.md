# seismic

Network latency & capacity monitoring system

## Sample output

This is a sample output for a 5-second transmission, with 1KiB chunks.

### Client

```
Measurements @ SystemTime { tv_sec: 1657841040, tv_nsec: 843414860 }
0.00s:        270 sent /          0 received
0.20s:        850 sent /         39 received
0.40s:        850 sent /        193 received
0.60s:       1210 sent /        311 received
0.80s:       1210 sent /        441 received
1.00s:       1210 sent /        604 received
1.20s:       1528 sent /        758 received
1.40s:       1909 sent /        930 received
1.60s:       1909 sent /       1130 received
1.80s:       2291 sent /       1332 received
2.00s:       2737 sent /       1551 received
2.20s:       2737 sent /       1787 received
2.40s:       3182 sent /       2033 received
2.60s:       3691 sent /       2276 received
2.80s:       3691 sent /       2544 received
3.00s:       4264 sent /       2825 received
3.20s:       4264 sent /       3079 received
3.40s:       4837 sent /       3340 received
3.60s:       4837 sent /       3600 received
3.80s:       5473 sent /       3852 received
4.00s:       5473 sent /       4112 received
4.20s:       6109 sent /       4362 received
4.40s:       6109 sent /       4629 received
4.60s:       6809 sent /       4872 received
4.80s:       6809 sent /       5130 received
5.00s:       6809 sent /       5380 received
5.20s:       6810 sent /       5643 received
5.40s:       6810 sent /       5906 received
5.60s:       6810 sent /       6164 received
5.80s:       6810 sent /       6417 received
6.00s:       6810 sent /       6681 received

```

![client-graph](assets/client-graph.png)

### Server

```
Measurements @ SystemTime { tv_sec: 1657841040, tv_nsec: 869478053 }
0.00s:          1 sent /          1 received
0.20s:         88 sent /         88 received
0.40s:        223 sent /        223 received
0.60s:        344 sent /        344 received
0.80s:        480 sent /        480 received
1.00s:        645 sent /        645 received
1.20s:        805 sent /        805 received
1.40s:        992 sent /        992 received
1.60s:       1184 sent /       1184 received
1.80s:       1400 sent /       1400 received
2.00s:       1618 sent /       1618 received
2.20s:       1847 sent /       1847 received
2.40s:       2092 sent /       2092 received
2.60s:       2341 sent /       2341 received
2.80s:       2618 sent /       2618 received
3.00s:       2897 sent /       2897 received
3.20s:       3152 sent /       3152 received
3.40s:       3404 sent /       3404 received
3.60s:       3669 sent /       3669 received
3.80s:       3919 sent /       3919 received
4.00s:       4175 sent /       4175 received
4.20s:       4427 sent /       4427 received
4.40s:       4687 sent /       4687 received
4.60s:       4940 sent /       4940 received
4.80s:       5200 sent /       5200 received
5.00s:       5452 sent /       5452 received
5.20s:       5713 sent /       5713 received
5.40s:       5972 sent /       5972 received
5.60s:       6232 sent /       6232 received
5.80s:       6485 sent /       6485 received
6.00s:       6748 sent /       6748 received

```

![server-graph](assets/server-graph.png)
