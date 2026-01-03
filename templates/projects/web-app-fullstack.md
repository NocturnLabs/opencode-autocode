<project_specification>
<project_name>{{PROJECT_NAME}}</project_name>

<overview>
A modern full-stack web application with {{DESCRIPTION}}.
Built with React frontend and Node.js/Express backend, featuring user authentication,
responsive design, and database persistence.
</overview>

<technology_stack>
<frontend>
<framework>React with Vite</framework>
<styling>Tailwind CSS</styling>
<state_management>Zustand</state_management>
<routing>React Router</routing>
</frontend>
<backend>
<runtime>Node.js with Express</runtime>
<database>SQLite with better-sqlite3</database>
<authentication>JWT with bcrypt</authentication>
</backend>
<communication>
<api>RESTful JSON API</api>
</communication>
</technology_stack>

<prerequisites>
<environment_setup>
- Node.js 18+ installed
- bun (preferred) or npm for package management
- .env file with JWT_SECRET
</environment_setup>
</prerequisites>

<core_features>
<user_authentication>

- User registration with email/password
- Login with JWT tokens
- Password hashing with bcrypt
- Protected routes
- Session persistence
  </user_authentication>

<responsive_ui>

- Mobile-first design
- Dark/light mode toggle
- Responsive navigation
- Toast notifications
  </responsive_ui>

<data_management>

- CRUD operations
- Form validation
- Optimistic updates
- Error handling
  </data_management>

<api_layer>

- RESTful endpoints
- Input validation
- Error responses
- Rate limiting
  </api_layer>
  </core_features>

<database_schema>
<tables>
<users>

- id, email, password_hash, created_at, updated_at
  </users>
  <sessions>
- id, user_id, token, expires_at
  </sessions>
  </tables>
  </database_schema>

<api_endpoints>
<auth>

- POST /api/auth/register (Create new user)
- POST /api/auth/login (Authenticate user)
- POST /api/auth/logout (Invalidate session)
- GET /api/auth/me (Get current user)
  </auth>
  <resources>
- GET /api/resources (List all)
- POST /api/resources (Create new)
- GET /api/resources/:id (Get one)
- PUT /api/resources/:id (Update)
- DELETE /api/resources/:id (Delete)
  </resources>
  </api_endpoints>

<implementation_steps>
<step number="1">

<title>Project Setup</title>
<tasks>
- Initialize Vite + React project
- Set up Express server in /server
- Configure SQLite database
- Set up environment variables
</tasks>
</step>

<step number="2">
<title>Authentication System</title>
<tasks>
- Create user model and migrations
- Implement registration endpoint
- Implement login with JWT
- Add auth middleware
</tasks>
</step>

<step number="3">
<title>Frontend Foundation</title>
<tasks>
- Set up Tailwind CSS
- Create layout components
- Implement routing
- Build auth forms
</tasks>
</step>

<step number="4">
<title>Core Features</title>
<tasks>
- Build CRUD UI components
- Connect to API endpoints
- Add loading states
- Implement error handling
</tasks>
</step>
</implementation_steps>

<success_criteria>
<functionality>

- Users can register and login
- Protected routes require authentication
- CRUD operations work correctly
- Data persists across sessions
  </functionality>
  <performance>
- Page load under 2 seconds
- API responses under 200ms
  </performance>
  </success_criteria>
  </project_specification>
