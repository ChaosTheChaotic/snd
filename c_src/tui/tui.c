#include "tui.h"
#include <ncurses.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int x, y;
static char **hostnames = NULL;
static int hostname_count = 0;
static int selected_index = 0;
static int scroll_offset = 0;
static const int LIST_HEIGHT = 10;

void termTUI() {
  // Free allocated hostnames
  if (hostnames) {
    for (int i = 0; i < hostname_count; i++) {
      free(hostnames[i]);
    }
    free(hostnames);
    hostnames = NULL;
  }
  endwin();
}

void setHostnames(char **hstnmes, int count) {
  // Free previous hostnames if any
  if (hostnames) {
    for (int i = 0; i < hostname_count; i++) {
      free(hostnames[i]);
    }
    free(hostnames);
  }

  hostname_count = count;
  hostnames = malloc(count * sizeof(char *));
  if (!hostnames) {
    endwin();
    fprintf(stderr, "Failed to malloc");
    exit(1);
  }

  for (int i = 0; i < count; i++) {
    hostnames[i] = strdup(hstnmes[i]);
  }
}

void drawUI() {
  clear();

  // Draw title
  attron(A_BOLD);
  mvprintw(1, (x - 50) / 2, "Network File Sharing - Host Selection");
  attroff(A_BOLD);

  // Draw instructions
  mvprintw(3, 2,
           "Use arrow keys or j/k to navigate, Enter to select, q to quit");

  // Draw list box
  box(stdscr, 0, 0);
  int list_width = x - 10;
  int list_x = 5;
  int list_y = 5;

  // Draw list border
  attron(A_BOLD);
  mvprintw(list_y - 1, list_x, "Available Hosts");
  attroff(A_BOLD);

  // Draw hostnames
  int visible_items =
      (y - list_y - 2 > LIST_HEIGHT) ? LIST_HEIGHT : y - list_y - 2;
  visible_items = (visible_items < hostname_count - scroll_offset)
                      ? visible_items
                      : hostname_count - scroll_offset;

  for (int i = 0; i < visible_items; i++) {
    int idx = i + scroll_offset;
    if (idx == selected_index) {
      attron(A_REVERSE); // Highlight selected item
    }
    mvprintw(list_y + i, list_x, " %-*s", list_width - 2, hostnames[idx]);
    if (idx == selected_index) {
      attroff(A_REVERSE);
    }
  }

  // Scroll indicators
  if (scroll_offset > 0) {
    mvprintw(list_y, list_x + list_width - 1, "↑");
  }
  if (scroll_offset + visible_items < hostname_count) {
    mvprintw(list_y + visible_items - 1, list_x + list_width - 1, "↓");
  }

  // Status bar
  attron(A_REVERSE);
  mvhline(y - 2, 1, ' ', x - 2);
  if (hostname_count > 0) {
    mvprintw(y - 2, 2, "Selected: %s (Press Enter to confirm)",
             hostnames[selected_index]);
  } else {
    mvprintw(y - 2, 2, "No hosts available");
  }
  attroff(A_REVERSE);

  refresh();
}

char *runTUI() {
  int ch;
  while (1) {
    ch = getch();
    refresh();
    switch (ch) {
    case 'q':
      termTUI();
      printf("Quit TUI");
      exit(0);
    case KEY_UP:
    case 'k':
      if (selected_index > 0) {
        selected_index--;
        // Adjust scroll position
        if (selected_index < scroll_offset) {
          scroll_offset = selected_index;
        }
      }
      break;
    case KEY_DOWN:
    case 'j':
      if (selected_index < hostname_count - 1) {
        selected_index++;
        // Adjust scroll position
        if (selected_index >= scroll_offset + LIST_HEIGHT) {
          scroll_offset = selected_index - LIST_HEIGHT + 1;
        }
      }
      break;
    case '\n':
      if (hostname_count > 0) {
        endwin();
        printf("Selected host: %s\n", hostnames[selected_index]);
        return strdup(hostnames[selected_index]);
      }
      break;
    }
    drawUI();
  }
  return NULL;
}

void initTUI() {
  initscr();
  cbreak();
  noecho();
  keypad(stdscr, TRUE);
  curs_set(0); // Hide cursor
  start_color();
  getmaxyx(stdscr, y, x);

  // Initialize with empty host list
  hostnames = NULL;
  hostname_count = 0;
  selected_index = 0;
  scroll_offset = 0;

  drawUI();
}
