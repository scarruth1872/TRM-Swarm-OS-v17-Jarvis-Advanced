# Continuum Dashboard

This project implements a "Continuum" dashboard, a monitoring interface for a distributed system, based on the provided mockup image. It is built using React, Tailwind CSS, and Recharts for data visualization.

## 🚀 Setup and Installation

1.  **Prerequisites**: Ensure you have Node.js (v18+) and npm (v9+) installed.

2.  **Clone/Download**: Place the provided files into a new directory.

3.  **Install Dependencies**:
    Navigate to the project root directory in your terminal and run:
    ```bash
    npm install
    ```

4.  **Run the Application**:
    Start the development server:
    ```bash
    npm start
    ```
    This will typically open the application in your browser at `http://localhost:3000`.

## ⚙️ Project Structure

```
.
├── public/
│   └── index.html             # Main HTML file
├── src/
│   ├── components/
│   │   ├── charts/
│   │   │   ├── NodeHealthChart.js
│   │   │   ├── ResourceUtilizationChart.js
│   │   │   └── TaskLatencyChart.js
│   │   ├── DashboardOverview.js
│   │   ├── EventLogStream.js
│   │   ├── Header.js
│   │   └── Sidebar.js
│   ├── App.js                 # Main application component
│   ├── index.css              # Tailwind CSS imports
│   └── index.js               # React entry point
├── package.json               # Project dependencies and scripts
├── tailwind.config.js         # Tailwind CSS configuration
└── postcss.config.js          # PostCSS configuration for Tailwind
```

## 📊 Data Sources

The dashboard currently uses static, mock data for demonstration purposes. In a production environment, these components would integrate with Swarm OS's High-Bandwidth Telemetry Fabric (HBTF) or other telemetry services to fetch real-time metrics via WebSockets or REST APIs.

## 🎨 Styling

Tailwind CSS is used for utility-first styling, providing a highly customizable and efficient way to build the UI.

## 📈 Charting

Recharts is utilized for rendering interactive and responsive data visualizations, including pie charts, line charts, and bar charts.