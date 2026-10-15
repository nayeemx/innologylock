# Prompt Log

Every prompt the user provides is recorded here with timestamps.
This file ensures session continuity — when a chat session is lost, the next AI reads this file to pick up from the last prompt.

---

## Session: 2026-08-24

| # | Time | Prompt |
|---|------|--------|
| 1 | 2026-08-24 | "scan entire project, read every .md files, find what was i working on just now, what are the prompt last time i provide? last cast session each and everything you able to find and solve them" |
| 2 | 2026-08-24 | "what .md file you maintain to record my prompt ever provided in here?" |
| 3 | 2026-08-24 | "i said record every prompt i said, it is importand, because before work if stored my prompt then next time tough the chat session is lost you can still work from last prompt" |
| 4 | 2026-08-24 | "in android you can keep files/software hidden, able to lock software/files on open it need password/finger on provide user able to access that, if that loch removed from software then able to access without password/fingure, this software you working on is for windows, but inpire by android you can easily create these features for windows easily, i want you to work on it. before work store my prompt so that next time you work on it you can identify what work what not work" |

## Session: 2026-08-25

| # | Time | Prompt |
|---|------|--------|
| 1 | 2026-08-25 | "what was you working on last time?" |
| 2 | 2026-08-25 | "scan the project read all the md files, find if you have something?" |
| 3 | 2026-08-25 | "tell me what this software can do?" |
| 4 | 2026-08-25 | "if you recorded yesterdays prompt based on that tell me what i want, based on that what this software serve what else remain, why that remain?" |
| 5 | 2026-08-25 | "can you solve them?" |
| 6 | 2026-08-25 | "work on remaining staff make it functional, if needed you can use this too: https://marketplace.visualstudio.com/items?itemName=Microsoft-WinAppCLI.winapp" |

## Session: 2026-08-26

| # | Time | Prompt |
|---|------|--------|
| 1 | 2026-08-26 | "What did we do so far?" |
| 2 | 2026-08-26 | "it has to run both in windows 10 and 11 too, to do that whatever you need just do it" |
| 3 | 2026-08-26 | "hey what doesn't work on windows 10 if the software run on windows 10 for that same feature whatever need to create separately you must create that. it is not mandatory same thing work on both os but what mandatory same feature has to work on both os for that separate engine needed for separate os then be it under same software." |
| 4 | 2026-08-26 | "Continue if you have next steps, or stop and ask for clarification if you are unsure how to proceed." |
| 5 | 2026-08-26 | "you mean everything is solved now? are you sure? what if i said i try to lock a folder but it doesn't able to lock that at all? whatever the feature it claimed as functional non of them working at all, not just that the software has unlimited bugs, user can't move menu to menu wile one pafe doing some work?" |
| 6 | 2026-08-26 | "hey, instate of [previous working title] i wantit to be as innologylock (from project folder to every single areas that has [previous working title] should change to innologylock even the github repository i have each and every areas where i see [it] has to be renamed to innologylock. but whatever you working or fixing this time. after that this rename should initiate" |
| 7 | 2026-08-26 | "are you sure? i can see folder name not yet change" |
| 8 | 2026-08-26 | "how about github repository name?" |
| 9 | 2026-08-26 | "what have you actually changed?" |
| 10 | 2026-08-26 | "what i have to do?" |
| 11 | 2026-08-26 | "have you update all the .md files? i will close this terminal on open i shall able to start chat from here, also i said to stored all my prompt in a separate .md file" |

## Session: 2026-08-27

| # | Time | Prompt |
|---|------|--------|
| 1 | 2026-08-27 | "What did we do so far?" |
| 2 | 2026-08-27 | "what you claim if failed i lose access to this folder and unable to delete it too. happens on my office pc before" |
| 3 | 2026-08-27 | "yes" (confirming safe ACL approach) |
| 4 | 2026-08-27 | "what failed?" |
| 5 | 2026-08-27 | "what it is?" (about panic hotkey error) |
| 6 | 2026-08-27 | "not actually another app i installed this app on my pc now testing on dev mode so that you understand what work what doesn't" |
| 7 | 2026-08-27 | "if permission issue then fix it" |
| 8 | 2026-08-27 | "seems like you stucked" |
| 9 | 2026-08-27 | [Provided full terminal output showing VaultPage UI + unlock widget] "you can see i already entered to the folder, and then the password window appear, what point of this? instate i should not entered to the folder that i locked and the password window appeared i input password then automatically entred the folder i select. isn't it? you keep failing on it. why?" |
| 10 | 2026-08-27 | "how long you need just to think instate of working actually? hey update everything you have + all the .md files, i will use another ai to fix this issues, and that ai should see all the current problem and working right from here. make sure it hable to be done" |

## Session: 2026-08-27 (continued)

| # | Time | Prompt |
|---|------|--------|
| 11 | 2026-08-27 | "scan theproject read all the .md files based on that work on this project + what i see from other ai" (sharing Claude Code's attempt at fixing ACL bug) |

## Session: 2026-10-15

| # | Time | Prompt |
|---|------|--------|
| 1 | 2026-10-15 | "scan entire project very deeply with whatever skills and power you have — ChatGPT scanned this project and reported a potential build-signing credential in README.md (signing-password value). Confirm whether that value is real or an example; if real, rotate it and assess whether the signing key itself was exposed. Also: the project must be innologylock everywhere — no trace of the [previous working title] name anywhere. Fix that too." (original wording redacted to remove old brand name per user's zero-tolerance rule) |
| 2 | 2026-10-15 | "things that if removed make the software unable to work or sacrifice its features are not acceptable — find another acceptable way to fix those" |
| 3 | 2026-10-15 | "have you updated all the .md files? that also contain what should i do? by the way what is signing password?" |
| 4 | 2026-10-15 | (rotation decision) "Explain first, decide later" — rotation procedure documented in BUILD_SETUP.md; no key changes made. Plus: "if new password stays in same location what's the point?" — answered: new password lives only in private places (password manager, git-ignored local file, GitHub Secrets); rotation kills the publicly-exposed history copies. |
| 5 | 2026-10-15 | "i like it, do it." — rotation attempted: built minisign re-encryption helper, but decrypt-test proved the key file matches NEITHER the recorded password NOR the original one (third unknown password). Blocked — handed masked-input step to user, awaiting "done". |
| 6 | 2026-10-15 | "up everything on github now" — committing + pushing all work. |

---

*Last updated: 2026-10-15*
