ALTER TABLE customers ADD COLUMN IF NOT EXISTS needs_review BOOLEAN DEFAULT FALSE NOT NULL;
ALTER TABLE customers ADD COLUMN IF NOT EXISTS review_reason TEXT;
CREATE INDEX IF NOT EXISTS idx_customers_needs_review ON customers(needs_review);
CREATE INDEX IF NOT EXISTS idx_customers_industry ON customers(industry);
