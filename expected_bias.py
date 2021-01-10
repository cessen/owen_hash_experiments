# The combination function.
def cmbn(n, k):
    if k == 0:
        return 1
    c = n
    for i in range(2, k + 1):
        n -= 1
        c *= n
        c //= i
    return c

# Expected bias in the average scramble tree at bit "bit", zero indexed.
def eb(bit):
    if bit <= 0:
        return 0.0
    elif bit > 15:
        print("You probably don't want to do a number that high--it \
               might make your compuer explode.")
        return None
    bit = 2**(bit - 1)
    precision = 1000000
    n = 0
    for i in range(0, bit + 1):
        n += cmbn(bit, i) * abs(precision * i // bit * 2 - precision)
    return (n // 2**bit) / precision


# Results up to bit 15, with precision 1/1,000,000:
# bit 0 = 0.0
# bit 1 = 1.0
# bit 2 = 0.5
# bit 3 = 0.375
# bit 4 = 0.273437
# bit 5 = 0.19638
# bit 6 = 0.139949
# bit 7 = 0.099346
# bit 8 = 0.070386
# bit 9 = 0.049819
# bit 10 = 0.035244
# bit 11 = 0.024927
# bit 12 = 0.017628
# bit 13 = 0.012466
# bit 14 = 0.008815
# bit 15 = 0.006233

# Estimated results up to bit 31, based on the ratio of subsequent
# numbers apparently converging to the square root of 0.5.
# bit 16 = 0.004407
# bit 17 = 0.003117
# bit 18 = 0.002204
# bit 19 = 0.001558
# bit 20 = 0.001102
# bit 21 = 0.000779
# bit 22 = 0.000551
# bit 23 = 0.000390
# bit 24 = 0.000275
# bit 25 = 0.000195
# bit 26 = 0.000138
# bit 27 = 0.000097
# bit 28 = 0.000069
# bit 29 = 0.000049
# bit 30 = 0.000034
# bit 31 = 0.000024


