# Format and Update Commit Messages

You should format your commit messages like this:

```
One-line title

Then a description, which may be multiple lines long.
This describes the change you are making with this commit.

Test Plan: how to test the change in this commit.

The test plan can also be several lines long.

Reviewers: github-username-a, github-username-b
```

The first line will be the title of the PR created by `spr diff`, and the rest of the lines except for the `Reviewers` line will be the PR description (i.e. the content of the first comment). The GitHub users named on the `Reviewers` line will be added to the PR as reviewers.

The `Test Plan` section is required to be present by default; `spr diff` will fail with an error if it isn't.
You can disable this in the [configuration](../reference/configuration.md).

## Updating the commit message

The local commit remains the source of truth after `spr diff` creates the PR. The local subject is the PR title. The local body is the PR description. Amend the local commit, then run `spr diff`. The command synchronizes metadata even when the code tree did not change.

Do not edit an SPR PR title or description directly in GitHub. If the user deliberately changed that metadata in GitHub and wants to keep it, run `spr amend` to import it into the local commit before the next diff.

`--update-message` remains accepted for compatibility, but it is not required. `--update-commit-message` and its `-m` short form only override the message of the synthetic update commit on the PR branch. They do not set the PR description.

## Further information

### Fields added by spr

At various stages of a commit's lifecycle, `spr` will add lines to the commit message:

- After first creating a PR, `spr diff` will amend the commit message to include a line like this at the end:

  ```
  Pull Request: https://github.com/example/project/pull/123
  ```

  The presence or absence of this line is how `spr diff` knows whether a commit already has a PR created for it, and thus whether it should create a new PR or update an existing one.

- `spr land` verifies that the local commit and PR are synchronized. It also adds a line like this:
  ```
  Reviewed By: github-username-a
  ```
  This line names the GitHub users who approved the PR.

### Example commit message lifecycle

This is what a commit message should look like when you first commit it, before running `spr` at all:

```
Add feature

This is a really cool feature! It's going to be great.

Test Plan:
- Run tests
- Use the feature

Reviewers: user-a, coworker-b
```

After running `spr diff` to create a PR, the local commit message will be amended to include a link to the PR:

```
Add feature

This is a really cool feature! It's going to be great.

Test Plan:
- Run tests
- Use the feature

Reviewers: user-a, coworker-b

Pull Request: https://github.com/example/my-thing/pull/123
```

In this state, running `spr diff` again will update PR 123.

Running `spr land` will verify the synchronized title and description, add the list of users who approved the PR, then land the commit. In this case, suppose only `coworker-b` approved:

```
Add feature

This is a really cool feature! It's going to be great.

Test Plan:
- Run tests
- Use the feature

Reviewers: user-a, coworker-b

Reviewed By: coworker-b

Pull Request: https://github.com/example/my-thing/pull/123
```

### Reformatting the commit message

spr is fairly permissive in parsing your commit message: it is case-insensitive, and it mostly ignores whitespace. You can run `spr format` to rewrite your HEAD commit's message to be in a canonical format.

This command does not touch GitHub; it doesn't matter whether the commit has a PR created for it or not.

Note that `spr land` will write the message of the commit it lands in the canonical format; you don't need to do so yourself before landing.
