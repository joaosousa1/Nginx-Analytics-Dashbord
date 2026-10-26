// Import CSS directly into JavaScript
import '../css/style.css';

document.addEventListener("DOMContentLoaded", () => {
    // Wait for the browser to perform the next visual rendering cycle
    requestAnimationFrame(() => {
        setTimeout(() => {
            //alert("hello rust");
        }, 0);
    });

});


import Chart from 'chart.js/auto';

// Global Configs Theme (Dark)
Chart.defaults.color = '#94a3b8';
Chart.defaults.borderColor = '#334155';

// Exposes the function globally so it can be called in the template
window.initDashboardCharts = function (visitsData, pathsData, statusData) {
    // 1. Gráfico de Visitas no Tempo
    const visitsCanvas = document.getElementById('visitsChart');
    if (visitsCanvas && visitsData) {
        new Chart(visitsCanvas, {
            type: 'line',
            data: {
                labels: visitsData.map(item => item.label),
                datasets: [{
                    label: 'Visitas',
                    data: visitsData.map(item => item.count),
                    borderColor: '#3b82f6',
                    backgroundColor: 'rgba(59, 130, 246, 0.15)',
                    fill: true,
                    tension: 0.3
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } }
            }
        });
    }

    // Top Point Chart
    const pathsCanvas = document.getElementById('pathsChart');
    if (pathsCanvas && pathsData) {
        new Chart(pathsCanvas, {
            type: 'bar',
            data: {
                labels: pathsData.map(item => item.label),
                datasets: [{
                    label: 'Pedidos',
                    data: pathsData.map(item => item.count),
                    backgroundColor: '#3b82f6',
                    borderRadius: 4
                }]
            },
            options: {
                indexAxis: 'y',
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } }
            }
        });
    }

    // Status Codes Chart
    const statusCanvas = document.getElementById('statusChart');
    if (statusCanvas && statusData) {
        const statusColors = {
            200: '#34d399',
            301: '#fbbf24',
            302: '#fbbf24',
            404: '#ef4444',
            500: '#dc2626'
        };

        new Chart(statusCanvas, {
            type: 'doughnut',
            data: {
                labels: statusData.map(item => 'HTTP ' + item.status),
                datasets: [{
                    data: statusData.map(item => item.count),
                    backgroundColor: statusData.map(item => statusColors[item.status] || '#94a3b8'),
                    borderWidth: 0
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { position: 'right' }
                }
            }
        });
    }
};
