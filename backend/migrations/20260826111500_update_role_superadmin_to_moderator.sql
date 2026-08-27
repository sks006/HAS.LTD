-- Update legacy user role mapping from 'SuperAdmin' to 'Moderator'
UPDATE users 
SET role = 'Moderator' 
WHERE role = 'SuperAdmin';
