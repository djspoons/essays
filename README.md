# Essays

See the GitHub Pages version of this repository.

# Installation

I mostly followed the directions listed here:

- https://docs.github.com/en/pages/setting-up-a-github-pages-site-with-jekyll/testing-your-github-pages-site-locally-with-jekyll
- https://jekyllrb.com/tutorials/using-jekyll-with-bundler/

Once you have Ruby installed, you should be able to test locally by running these commands:

```
bundle install
bundle exec jekyll serve
```

# Managing the local copy Tuun web

In the Tuun repo, create a subtree split with branch `web-split` and push it.

```sh
git subtree split --prefix=web -b web-split
git push -u origin web-split
```

Then in this repo, add a new remote for the Tuun repo:
```sh
git remote add origin-tuun git@github.com:djspoons/tuun.git
```

To update the local copy run the following:
```sh
git subtree add --prefix=tuun --squash origin-tuun web-split
```

And remember to git push as well!
```sh
git push
```
