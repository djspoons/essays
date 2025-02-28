

A3 = [220 * f for f in [1, 2, 3, 4]]
A4 = [440 * f for f in [1, 2]]
E4 = [329.6 * f for f in [1, 2, 3]]
B3 = [246.9 * f for f in [1, 2, 3, 4]]
Asharp3 = [233.1 * f for f in [1, 2, 3, 4]]
Eflat4 = [311.1 * f for f in [1, 2, 3, 4]]

def ERB(freq):
    return freq / 9.2645 + 24.7


def min_diff(freq, other):
    min = 22000
    other_freq = 0
    for f in other:
        if abs(freq - f) < min:
            min = abs(freq - f)
            other_freq = f
    return (min, other_freq)

print("A3 vs A4")
for freq in A3:
    (min, other_freq) = min_diff(freq, A4)
    erb = ERB((freq + other_freq) / 2)
    print("freq: {0:.1f}, other: {1:.1f}, min diff: {2:.1f}, erb: {3:.1f}, % of erb: {4:.0f}\n".format(freq, other_freq, min, erb, min / erb * 100))


print("A3 vs E4")
for freq in A3:
    (min, other_freq) = min_diff(freq, E4)
    erb = ERB((freq + other_freq) / 2)
    print("freq: {0:.1f}, other: {1:.1f}, min diff: {2:.1f}, erb: {3:.1f}, % of erb: {4:.0f}\n".format(freq, other_freq, min, erb, min / erb * 100))

""" print("A3 vs B3")
for freq in A3:
    (min, other_freq) = min_diff(freq, B3)
    erb = ERB((freq + other_freq) / 2)
    print("freq: {}, other: {}, min diff: {}, erb: {}, % of erb: {}\n".format(freq, other_freq, min, erb, min / erb * 100))
 """

print("A3 vs Eb4")
for freq in A3:
    (min, other_freq) = min_diff(freq, Eflat4)
    erb = ERB((freq + other_freq) / 2)
    print("freq: {0:.1f}, other: {1:.1f}, min diff: {2:.1f}, erb: {3:.1f}, % of erb: {4:.0f}\n".format(freq, other_freq, min, erb, min / erb * 100))


