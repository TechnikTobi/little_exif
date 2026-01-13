# How to (somewhat) successfully reduce the repo size
As part of PR #91, I wanted to shrink the size of the repository so that 
fetching it in the future won't require downloading blobs with old, large 
image files that have been replaced with smaller, compressed versions. 

## Initial Steps
First, I had the general problem of reducing size with these steps

### Install tools
Install `git-filter-repo`
```bash
brew install git-filter-repo
```

### Make a backup
As rewriting the history is destructive, keeping a backup in case something 
needs to be restored is a good idea
```bash
git clone --mirror https://github.com/TechnikTobi/little_exif.git l-e-backup.git
```

### Remove files from history
Change directory to the actual repo (not the backup!).
Delete all files above a certain size (this is what I used)
```bash
git filter-repo --strip-blobs-bigger-than 1M
```

Or, you could also remove specific files: (not used)
```bash
git filter-repo --path path/to/large_image.png --invert-paths
```

Also, remove all files with a specific extension: (not used)
```bash
git filter-repo \
  --path-glob '*.png' \
  --path-glob '*.jpg' \
  --invert-paths
```

There likely will be a notice saying something about "removing 'origin' 
remote", which is expected behavior. 

### Re-add Remote & Push
Check that no remote exists (this should print nothing)
```bash
git remote -v
```

Re-add the GitHub remote (can be verified with the command above)
```bash
git remote add origin https://github.com/TechnikTobi/little_exif.git
```

Force push the cleaned history
```bash
git push --force --all origin
git push --force --tags origin
```

## Side-effect handling: Tags of commits are "lost"
Technically, they are not lost! They still exist but point to old commit IDs,
as rewriting the history also changed *all* commit IDs!
This is also a possible reason why the repository on GitHub is still the size
it was before, as the garbage collection can't clean up the old commits due
to the tags still pointing to them.
Unfortunately, there is currently no known way that works for "re-mapping"
tags to the new commits. So, in order to fully resize the repo, the tags 
needed to be removed locally & remotely

### Remove the local tags
Can't fully remember if this is 100% required/if this command even works,
but I did run this prior according to the history, so here you go:
```bash
git tag -l | xargs git tag -d
```

### Remove the remote tags
Print the tags still on GitHub:
```bash
git ls-remote --tags origin
```

Delete all remote tags, which may print errors for tags with the suffix
"^{}" - these errors can be ignored. 
```bash
git ls-remote --tags origin \
  | awk '{print $2}' \
  | sed 's|refs/tags/||' \
  | xargs -n 1 git push origin --delete
```

Verify that no remote tags exist anymore by re-running the previous command.

Optional: Perform one final force-push:
```bash
git push --force --all origin
```
