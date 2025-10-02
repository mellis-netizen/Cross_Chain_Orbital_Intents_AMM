#!/usr/bin/env node

const fs = require('fs')
const path = require('path')
const { exec } = require('child_process')
const util = require('util')

const execAsync = util.promisify(exec)

// ANSI color codes
const colors = {
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  reset: '\x1b[0m',
  bold: '\x1b[1m'
}

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`)
}

function logSuccess(message) {
  log(`✅ ${message}`, 'green')
}

function logError(message) {
  log(`❌ ${message}`, 'red')
}

function logWarning(message) {
  log(`⚠️  ${message}`, 'yellow')
}

function logInfo(message) {
  log(`ℹ️  ${message}`, 'blue')
}

async function checkNodeModules() {
  logInfo('Checking node_modules...')
  
  const nodeModulesPath = path.join(__dirname, 'node_modules')
  if (!fs.existsSync(nodeModulesPath)) {
    logError('node_modules directory not found')
    logInfo('Run: npm install')
    return false
  }
  
  // Check for key dependencies
  const keyDeps = ['next', 'react', 'wagmi', 'viem', 'tailwindcss']
  for (const dep of keyDeps) {
    const depPath = path.join(nodeModulesPath, dep)
    if (!fs.existsSync(depPath)) {
      logError(`Missing dependency: ${dep}`)
      return false
    }
  }
  
  logSuccess('All key dependencies found')
  return true
}

async function checkBuildFiles() {
  logInfo('Checking for build output...')
  
  const buildPath = path.join(__dirname, '.next')
  if (!fs.existsSync(buildPath)) {
    logWarning('No build output found (.next directory missing)')
    logInfo('Run: npm run build')
    return false
  }
  
  logSuccess('Build output found')
  return true
}

async function checkConfigFiles() {
  logInfo('Checking configuration files...')
  
  const requiredFiles = [
    'package.json',
    'next.config.js',
    'tailwind.config.js',
    'tsconfig.json',
    'postcss.config.js'
  ]
  
  let allFound = true
  for (const file of requiredFiles) {
    const filePath = path.join(__dirname, file)
    if (!fs.existsSync(filePath)) {
      logError(`Missing config file: ${file}`)
      allFound = false
    }
  }
  
  if (allFound) {
    logSuccess('All configuration files found')
  }
  
  return allFound
}

async function checkSourceFiles() {
  logInfo('Checking source files...')
  
  const requiredDirs = [
    'src/app',
    'src/components',
    'src/hooks',
    'src/lib',
    'src/types',
    'src/utils'
  ]
  
  let allFound = true
  for (const dir of requiredDirs) {
    const dirPath = path.join(__dirname, dir)
    if (!fs.existsSync(dirPath)) {
      logError(`Missing source directory: ${dir}`)
      allFound = false
    }
  }
  
  if (allFound) {
    logSuccess('All source directories found')
  }
  
  return allFound
}

async function checkPackageScripts() {
  logInfo('Checking package.json scripts...')
  
  const packagePath = path.join(__dirname, 'package.json')
  if (!fs.existsSync(packagePath)) {
    logError('package.json not found')
    return false
  }
  
  const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf8'))
  const requiredScripts = ['dev', 'build', 'start', 'lint']
  
  let allFound = true
  for (const script of requiredScripts) {
    if (!packageJson.scripts?.[script]) {
      logError(`Missing script: ${script}`)
      allFound = false
    }
  }
  
  if (allFound) {
    logSuccess('All required scripts found')
  }
  
  return allFound
}

async function testBuild() {
  logInfo('Testing build process...')
  
  try {
    const { stdout, stderr } = await execAsync('npm run build', {
      cwd: __dirname,
      timeout: 60000 // 1 minute timeout
    })
    
    if (stderr && !stderr.includes('warn')) {
      logWarning(`Build warnings: ${stderr}`)
    }
    
    logSuccess('Build completed successfully')
    return true
  } catch (error) {
    logError(`Build failed: ${error.message}`)
    if (error.stdout) {
      console.log('Build output:', error.stdout)
    }
    if (error.stderr) {
      console.log('Build errors:', error.stderr)
    }
    return false
  }
}

async function checkTypeScript() {
  logInfo('Checking TypeScript compilation...')
  
  try {
    const { stdout, stderr } = await execAsync('npm run type-check', {
      cwd: __dirname,
      timeout: 30000 // 30 seconds timeout
    })
    
    logSuccess('TypeScript compilation successful')
    return true
  } catch (error) {
    logError(`TypeScript errors found: ${error.message}`)
    return false
  }
}

async function checkEnvironmentVariables() {
  logInfo('Checking environment variables...')
  
  const envPath = path.join(__dirname, '.env.local')
  if (!fs.existsSync(envPath)) {
    logWarning('.env.local file not found')
    logInfo('Create .env.local with required environment variables')
    return false
  }
  
  const envContent = fs.readFileSync(envPath, 'utf8')
  const requiredVars = [
    'NEXT_PUBLIC_CHAIN_ID',
    'NEXT_PUBLIC_RPC_URL',
    'NEXT_PUBLIC_NETWORK_NAME'
  ]
  
  let allFound = true
  for (const varName of requiredVars) {
    if (!envContent.includes(varName)) {
      logWarning(`Missing environment variable: ${varName}`)
      allFound = false
    }
  }
  
  if (allFound) {
    logSuccess('All environment variables configured')
  }
  
  return allFound
}

async function checkDeploymentReadiness() {
  logInfo('Checking deployment readiness...')
  
  // Check for deployment files
  const deploymentFiles = [
    'netlify.toml',
    'deploy-netlify.sh'
  ]
  
  let deploymentReady = true
  for (const file of deploymentFiles) {
    const filePath = path.join(__dirname, file)
    if (!fs.existsSync(filePath)) {
      logWarning(`Deployment file not found: ${file}`)
      deploymentReady = false
    }
  }
  
  if (deploymentReady) {
    logSuccess('Deployment configuration found')
  }
  
  return deploymentReady
}

async function generateReport() {
  log('\\n' + '='.repeat(60), 'bold')
  log('🚀 ORBITAL AMM FRONTEND VERIFICATION REPORT', 'bold')
  log('='.repeat(60), 'bold')
  
  const checks = [
    { name: 'Node Modules', fn: checkNodeModules },
    { name: 'Configuration Files', fn: checkConfigFiles },
    { name: 'Source Files', fn: checkSourceFiles },
    { name: 'Package Scripts', fn: checkPackageScripts },
    { name: 'Environment Variables', fn: checkEnvironmentVariables },
    { name: 'TypeScript Compilation', fn: checkTypeScript },
    { name: 'Build Process', fn: testBuild },
    { name: 'Deployment Readiness', fn: checkDeploymentReadiness }
  ]
  
  const results = []
  
  for (const check of checks) {
    log(`\\n📋 ${check.name}...`, 'blue')
    try {
      const result = await check.fn()
      results.push({ name: check.name, success: result })
    } catch (error) {
      logError(`Error in ${check.name}: ${error.message}`)
      results.push({ name: check.name, success: false })
    }
  }
  
  // Summary
  log('\\n' + '='.repeat(60), 'bold')
  log('📊 SUMMARY', 'bold')
  log('='.repeat(60), 'bold')
  
  const passed = results.filter(r => r.success).length
  const total = results.length
  const percentage = Math.round((passed / total) * 100)
  
  results.forEach(result => {
    if (result.success) {
      logSuccess(result.name)
    } else {
      logError(result.name)
    }
  })
  
  log(`\\n📈 Score: ${passed}/${total} (${percentage}%)`, 'bold')
  
  if (percentage >= 80) {
    logSuccess('Frontend is ready for deployment! 🎉')
  } else if (percentage >= 60) {
    logWarning('Frontend needs some fixes before deployment')
  } else {
    logError('Frontend requires significant work before deployment')
  }
  
  // Quick start instructions
  log('\\n' + '='.repeat(60), 'bold')
  log('🚀 QUICK START', 'bold')
  log('='.repeat(60), 'bold')
  
  log('1. Install dependencies:', 'blue')
  log('   npm install')
  
  log('\\n2. Start development server:', 'blue')
  log('   npm run dev')
  
  log('\\n3. Build for production:', 'blue')
  log('   npm run build')
  
  log('\\n4. Deploy to Netlify:', 'blue')
  log('   ./deploy-netlify.sh')
  
  log('\\n5. Access application:', 'blue')
  log('   http://localhost:3000')
  
  return percentage >= 80
}

// Run verification
if (require.main === module) {
  generateReport()
    .then(success => {
      process.exit(success ? 0 : 1)
    })
    .catch(error => {
      logError(`Verification failed: ${error.message}`)
      process.exit(1)
    })
}

module.exports = { generateReport }