// Well apparently the idea I had was doing everything wrong, I was trying an
// approach with ntfw and a simple counter that worked with the stat command.
#define _XOPEN_SOURCE 500
#include <ftw.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>

typedef struct {
  dev_t dev;
  ino_t ino;
} inode_key;

static __thread unsigned long long int total = 0;
static __thread inode_key *seen_inodes = NULL;
static __thread size_t seen_count = 0;
static __thread size_t seen_capacity = 0;

int sum(const char *fpath, const struct stat *sb, int tflag,
        struct FTW *ftwbuf) {
  // Skip unreadable/unstatable files
  if (tflag == FTW_DNR || tflag == FTW_NS) {
    fprintf(stderr, "Skipping inaccessible: %s\n", fpath);
    return 0;
  }

  // Handle directories and symlinks
  if (tflag == FTW_D || tflag == FTW_DP) {
    total += sb->st_blocks * 512;
    return 0;
  }

  // Check for hard links (files/symlinks only)
  bool already_counted = false;
  for (size_t i = 0; i < seen_count; i++) {
    if (seen_inodes[i].dev == sb->st_dev && seen_inodes[i].ino == sb->st_ino) {
      already_counted = true;
      break;
    }
  }

  if (!already_counted) {
    // Add new inode to tracking
    if (seen_count >= seen_capacity) {
      seen_capacity = seen_capacity ? seen_capacity * 2 : 64;
      seen_inodes = realloc(seen_inodes, seen_capacity * sizeof(inode_key));
      if (!seen_inodes) {
        perror("realloc");
        exit(EXIT_FAILURE);
      }
    }
    seen_inodes[seen_count] = (inode_key){sb->st_dev, sb->st_ino};
    seen_count++;

    // Count disk space
    total += sb->st_blocks * 512;
  }
  return 0;
}

unsigned long long int du(char path[], bool fsym) {
  // Reset state for new traversal
  total = 0;
  seen_count = 0;

  if (!path) {
    fprintf(stderr, "du called without valid path\n");
    exit(EXIT_FAILURE);
  }

  struct stat sbuf;
  int (*sfunc)(const char *, struct stat *) = fsym ? stat : lstat;
  if (sfunc(path, &sbuf) == -1) {
    perror("stat/lstat");
    exit(EXIT_FAILURE);
  }

  if (!S_ISDIR(sbuf.st_mode)) {
    return sbuf.st_blocks * 512;
  }

  int flags = fsym ? 0 : FTW_PHYS;
  if (nftw(path, sum, 20, flags) == -1) {
    perror("nftw");
    exit(EXIT_FAILURE);
  }
  return total;
}
