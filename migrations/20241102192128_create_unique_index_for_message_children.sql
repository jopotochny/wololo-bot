-- Add migration script here
ALTER TABLE message_children
    ADD CONSTRAINT UQ_parent_child UNIQUE(parent, child)