# Spec Delta

## Purpose

Defines how animations of a sprite are created, shown, edited and taught to AI agents.

## ADDED Requirements

### Requirement: Animations do not overlap
An animation SHALL be a tag whose frames do not belong to another animation of the same sprite.

#### Scenario: Overlap refused
- **WHEN** a tool or the editor creates an animation that shares a frame with another one
- **THEN** it is refused with the code `animation_overlap` and nothing changes

### Requirement: Create an animation from the base
`create_animation` SHALL copy a base frame the requested number of times at the end of the sprite, tag the new frames with the name, fps and mode, and do it as one history entry without sending pixels.

#### Scenario: Four frames from the base
- **WHEN** an agent calls `create_animation` with name `walk`, 4 frames and fps 6 on a one-frame sprite
- **THEN** the sprite has 5 frames, frames 2 to 5 equal frame 1, and `walk` is a loop at 6 fps over frames 2 to 5

#### Scenario: Name already used
- **WHEN** the name exists, ignoring case
- **THEN** the call fails with `name_exists` and nothing changes

### Requirement: Manage animations
`duplicate_animation`, `rename_animation`, `delete_animation` and `move_frame` SHALL change animations with one history entry each. `delete_animation` SHALL keep the frames unless `with_frames` is true.

#### Scenario: Delete with frames
- **WHEN** an animation is deleted with `with_frames` true
- **THEN** its frames are removed and the tags after it follow their frames

### Requirement: Region tools on many frames
`transform_region` and `copy_region` SHALL accept `frames` (a list or a range such as `"8-13"`) and apply the same operation to every frame as one atomic history entry.

#### Scenario: One call, six frames
- **WHEN** `transform_region` moves a region on frames `"8-13"`
- **THEN** the six frames change together and one undo restores all of them

### Requirement: Named parts
A sprite SHALL accept named rectangles (parts) set with `set_part` and removed with `remove_part`, and `transform_region` and `copy_region` SHALL accept `part` instead of `region`. Old projects open without parts.

#### Scenario: Move the head on many frames
- **WHEN** `transform_region` uses `part` "head", `dy` 3 and `frames` "8-13"
- **THEN** the pixels inside the rectangle of that part move on those frames

### Requirement: The frames strip is grouped by animation
The editor SHALL group the frames by animation with a collapsible header (name, fps, loop, play), a "Base" group first and a "No animation" group last, and SHALL offer "New animation..." (name, frames, fps, mode) and, in the header menu, rename, duplicate and delete (with or without frames), each as one undo step.

#### Scenario: Two animations
- **WHEN** a sprite has the animations `walk` and `jump`
- **THEN** the strip shows Base, walk and jump as groups, and playing a group plays only its frames

### Requirement: Rules for agents
The server instructions, the guide topic `animations-from-base` and the Claude Code skill SHALL tell agents to draw the base first, make one named animation per action from the base, and NEVER generate pixels outside the app (scripts or image libraries) without the agreement of the person.

#### Scenario: Skill and server agree
- **WHEN** the tests compare the skill with the server guide
- **THEN** both carry the same rules and every tool named exists
