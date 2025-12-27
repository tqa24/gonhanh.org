-- Personalization Engine Database Schema

-- Gorse database (recommendation engine)
CREATE DATABASE IF NOT EXISTS gorse;
GRANT ALL ON gorse.* TO 'gorse'@'%';

-- Use personalization database
USE personalization;

-- Lead Scores Table
CREATE TABLE IF NOT EXISTS lead_scores (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id VARCHAR(255) NOT NULL,
    score DECIMAL(5,2) NOT NULL,
    segment VARCHAR(50) NOT NULL,
    factors JSON NOT NULL,
    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_user_id (user_id),
    INDEX idx_segment (segment),
    INDEX idx_calculated_at (calculated_at)
);

-- Content Rules Table
CREATE TABLE IF NOT EXISTS content_rules (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    conditions JSON NOT NULL,
    actions JSON NOT NULL,
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_is_active (is_active),
    INDEX idx_priority (priority)
);

-- A/B Experiments Table
CREATE TABLE IF NOT EXISTS ab_experiments (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    variants JSON NOT NULL,
    traffic_allocation JSON NOT NULL,
    status ENUM('draft', 'running', 'paused', 'completed') DEFAULT 'draft',
    start_at TIMESTAMP NULL,
    end_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_status (status)
);

-- Customers Table (CDP mirror)
CREATE TABLE IF NOT EXISTS customers (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255),
    data JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_user_id (user_id),
    INDEX idx_email (email)
);

-- Insert sample content rule
INSERT INTO content_rules (name, description, conditions, actions, priority, is_active) VALUES
(
    'Show Demo CTA for Hot Leads',
    'Display demo request CTA for users with lead score >= 50',
    '[{"field": "lead_score", "operator": "greater_than", "value": 50}]',
    '[{"show_content": {"content_id": "demo-cta-banner"}}]',
    100,
    TRUE
),
(
    'Pricing Page Discount for Warm Leads',
    'Show 10% discount banner on pricing page for warm leads',
    '[{"field": "segment", "operator": "in", "value": ["warm", "hot"]}, {"field": "page_url", "operator": "contains", "value": "pricing"}]',
    '[{"show_content": {"content_id": "discount-banner-10"}}]',
    90,
    TRUE
);
