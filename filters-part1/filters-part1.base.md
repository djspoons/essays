# Filters ~~for~~ from a Dummy, Part 1

Some reflections on digital audio filters, because 1) I needed to implement filters as part of building a software synthesizer and 2) analogies to electric circuits or mechanical springs mean almost nothing to me.  Hopefully these will help provide some intuitions for you. In part one: feed-forward filters, aka finite impulse response filters, aka filtering by convolution. Some reflections in which I'm awestruck by what a little addition and multiplication can get you.

## Background

By audio filter, I mean a transformation that amplifies or attenuates certain frequencies. You can think of this like the "EQ" control on your stereo amplifier or other playback device. When you adjust the "treble" or "bass," you are applying a filter.

Filters are important because we might want to filter out noise (that is, attenuate high frequencies) that was introduced in a live recording. Or remove low frequencies that sound unpleasant when played in a small room. Filters are also critical in _subtractive audio synthesis_ where different instruments are created by starting with a waveform that contains many frequencies (for example, a square wave) and then filtering out frequencies selectively.

As I'm writing audio processing _software_, I'm interested in _digital_ audio filters: mathematical functions that transform audio samples to boost or reduce certain frequencies. At its simplest, a digital audio filter gets one input sample at a time (and some context) and generates a corresponding output sample.

> Caveat: I'm going to focus on filtering the samples -- that is, in the _time_ domain, not the _frequency_ domain -- as I'm interested in doing filtering in real time. Don't worry! The frequency domain will appear soon enough.

There's one intuition that, like a sieve, a filter lets some things through while blocking others. At a high level, this is a good starting point, and maybe you can think of a sieve with small holes only letting high frequencies through, and one with big holes letting more frequencies though.  For me though, that doesn't help _that_ much, since I'm implementing a filter that operates at the level of individual samples: what does it mean to let through part of a sample?

I'm going to focus on "low-pass" filters, those that let low frequencies pass through unchanged and reduce the amplitude of higher frequencies. I find these filters relatively easy to think about. One important parameter of a low-pass filter is its "cut-off frequency" -- the frequency below which signal is passed through and above which it is not. While an ideal low-pass filter can be described only in terms of this cutoff frequency, realistic filters don't behave quite so cleanly. For example, many low-pass filters also have a transition band -- a range of frequencies where attenuation begins and then reaches its maximum.

## Feed-forward Filters

I wrote above that a filter is a function that takes each sample and some context and then generates a new sample. For feed-forward filters, this context is some of the previously seen input samples. In other words, input samples are fed _forward_ in time and can affect both the current output sample and future output samples. Or put another way, an input sample can be _delayed_ and act as an input for future output samples.

### Moving Average Filter

Let's start with a most naive of low-pass filters, the simple moving average filter. It's "simple" because it takes a fixed number of samples and weights them all equally. The "order" of a feed-forward filter is defined as the number of previous input samples that can be used as part of computing the current output sample. The "length" is the total number of input samples used, that is, one more than the order. A third-order (length = 4) moving average filter looks like this:

$$
  y[n] = \frac{x[n] + x[n-1] + x[n-2] + x[n-3]}{4}
$$

Where $x$ represents the input samples and $y$ is the output. This filter replaces every sample with the average of the last four samples, including the current one. In doing so, it smooths out the waveform: it reduces the amount of change from one sample to the next. What does it mean when there is a lot of change from one sample to the next? That occurs when there are higher frequencies present in the waveform, so smoothing means that higher frequencies are attenuated.

#### Removing Noise

One use of a low-pass filter is to remove noise from a signal. If we take a pure tone and add noise to it (as you can hear in the first example below), we can use a 15th-order moving average filter to remove some of that noise (as you can hear in the second).
```tuun -C context.tuun
($220 + noise * 0.25) * 0.75 | fin(time - 5) | capture("01-moving-avg-noise-orig") | moving_average(15) | capture("02-moving-avg-noise-ma15")
```
This noise is uniform across the range of frequencies (that is, it has energy at all frequencies) so a low-pass filter won't remove all of it, just the noise at higher frequencies.

#### Removing Clicks

A "click" is another type of distortion that you might like to remove from an audio signal. A moving average filter is not especially good at removing this sort of distortion, but it's useful to understand why. The waveform below is a pure tone combined with a 1 Hz click, both without and with the same moving average filter.
```tuun -C context.tuun
$220 + res($1, fixed([1,1,1])) | fin(time - 5) | capture("01-moving-avg-clicks-orig") | moving_average(15) | capture("02-moving-avg-clicks-ma15")
```
One way to think about this case is that response of the filter is very "sharp" -- samples move into the window and immediately carry the full weight of every other sample that's already in the window. Likewise, when they move out of the window, they immediately have no effect on the output at all. This means that a moving average filter is unable to remove short but abrupt distortions like these clicks: it can smooth them out a little bit, but they are still quite audible.

#### Removing a Pure Tone

Since we'd like to better understand how the moving average filter affects different frequencies, let's use waveforms comprised of only two pure tones -- just two frequencies. The first waveform you can hear below is comprised of two component tones without a filter, while the second has the 15th-order moving average filter applied.
```tuun -C context.tuun
($220 + $2560) * 0.6 | fin(time - 5) | capture("01-moving-avg-2tones-orig") | moving_average(15) | capture("02-moving-avg-2tones-ma15")
```

While the higher frequency tone is still present in the second waveform, it has been significantly attenuated.

Moving average filters have some surprising (to me) behavior. This waveform uses the same filter and starts with the same two tones, but the second tone is gradually increased in frequency.
```tuun -C context.tuun
($220 + $(linear(2560, 40))) * 0.6 | fin(time - 6) | moving_average5) | capture("01-moving-avg-2tones-changing")
```
What happened here? Ideally, a low-pass filter would attenuate more as the frequency got higher (or at least _not attenuate less_), but here the higher tone first gets softer and then louder. That is, the gain of the filter goes down **and then up** as the frequency increases. This is called a "ripple." We want to avoid ripples in low-pass filters in many situations, including (for example) in music, where melodies would be distorted as they pass across ripples.

Remember that each tone is periodic and that for each negative value, an equal positive value appears half a wavelength later. If the number of elements in the moving average is equal to a multiple of the wavelength of the waveform (in terms of the number of samples), then these elements will cancel each other out. A 15th-order moving average has 16 elements in it, and 1/16th of the sampling frequency (44100 Hz) is 2756 Hz, which is the frequency of the tone when it seemed to disappear.

A final challenge with a moving average filter is that we don't have much control over the cutoff frequency. We can increase the order of the filter -- this will lower the cutoff frequency -- but it will also add ripples. In addition, the transition band is quite wide, and there's no way to control it.

### Analyzing Feed-forward Filters

Above I wrote the moving average filter as you might expect an average to be written, but let's pull out the weights used in computing the average, which here are all equal. Here is the third order filter express as those weights:
$$
h = \left[ \frac{1}{4},\frac{1}{4},\frac{1}{4},\frac{1}{4} \right]
$$
Once we've done that, we can now write a generic equation for a feed-forward filter like this:
$$
y[n] = \sum_0^{K-1} h[k] \cdot x[n - k] 
$$
(Remember we are assuming that $x$ is periodic, so we can just repeat the waveform to determine $x[n]$ for negative values of $n$.)


$$
y[n] = (h * x)[n] 
$$
Where $*$ is the convolution operator: it returns the sum 
<!--

define convolution 

... and spectrum

impulse response?


> Feed-forward filters are often called "finite impulse response" filters because, if the input samples eventually go to zero, then the output of the filter will also go to zero within a finite number of samples. The simplest case of an input that "goes to zero" is an impulse: a waveform where the first sample is `1` and all of the remaining samples are `0`.


-->


#### Frequency Response

Grab a pencil and as you listen to the following waveform, plot the _loudness_ of what you hear.
```tuun -C context.tuun
// maybe (really) shouldn't be linear
$(linear(100, 500)) | fin(time - 20) | moving_average(15) | capture("01-moving-avg-1tone-changing")
```
What you probably heard is a tone that starts loud and -- as its frequency increases -- gets soft, then oscillates between loud and soft. This is the "frequency response" of the filter. That is, we passed one frequency at a time through the filter, and you measured the gain or attenuation at that frequency. Though you made the graph for just one frequency at a time, this graph can be used to understand how the filter will respond to any combination of frequencies.

#### The Convolution Revolution



Here are two things you need to buy into:

1. The discrete Fourier transform (DFT) transforms periodic signals expressed as a function of time (that is, as a series of samples) into signals expressed as a combination of sinusoids at different frequencies, called a _spectrum_. That is, it breaks down one waveform into the contributions of a number of simple waveforms. These two representations are equivalent: either can be used to faithfully recreate the other.

2. In addition to using convolution in the time domain (for example, the weighted average above), feed-forward filters can be implemented by _element-wise multiplication_ in the frequency domain. That is, if we take the DFT of the signal and the DFT of the filter and multiply the elements together, the result will be the DFT of the convolution of the original signal and filter. This is called the "convolution theorem."

Taking the first: it should be reasonably clear that we can go in the inverse direction -- that we can take two or more sinusoids and, if we combine them with the proper weights, recreate the original waveform. How does the DFT start with the waveform and find the right combination of sinusoids? XXX

The second point is a little more subtle, though you've already seen part of it in action yourself. When you when you listened to the moving average filter applied to two tones and when you plotted its frequency response, you showed that the filter can be applied independently to each component frequency. XXX

And in fact, if $x = x_1 + x_2$ then it's easy to show that we can apply the convolution to each component and then add the results. (This is just distributing multiplication over addition.)

$$
y[n] = \sum_0^{K-1} h[k] \cdot x[n - k] = \sum_0^{K-1} h[k] \cdot (x_1[n - k] + x_2[n - k])
$$
$$
  = \sum_0^{K-1} (h[k] \cdot x_1[n - k]) + (h[k] \cdot x_2[n - k])
$$
$$
  = \sum_0^{K-1} h[k] \cdot x_1[n - k] + \sum_0^{K-1} h[k] \cdot x_2[n - k]
$$

What about the DFT of the filter? XXX


But really covering these properly is more than I can do in a short piece like this, so I highly recommend 
[Brian McFee's Digital Signals Theory](https://brianmcfee.net/dstbook-site/content/intro.html) which presents both of these in detail and with some wonderful interactive plots. 

(Two quick notes: I should say that I'm glossing over the phase of the component sinusoids, which the DFT should and does account for. And secondly, the convolution theorem offers a potentially faster way of applying a filter, by converting the signal into the frequency domain, multiplying, and the applying the inverse discrete Fourier transform.)

The important bit for the current discussion is that we can perform a DFT of the filter itself to understand which frequencies it will boost and which it will reduce.



### Windowed Sinc Filter

Rather than 

A better method is to use a series of _unequal_ weights.

<!--


as delay

-->
